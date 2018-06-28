extern crate gb_emu;
extern crate sdl2;

use gb_emu::Emulator;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

struct FrameTimer {
    start: SystemTime,
    frame_time: u32,
}

impl FrameTimer {
    fn new(frame_time_nanos: u32) -> FrameTimer {
        FrameTimer {
            start: SystemTime::now(),
            frame_time: frame_time_nanos,
        }
    }

    fn sleep_till_end_of_frame(&self) {
        let frame_time = Duration::new(0, self.frame_time);
        let sleep_time = match self.start.elapsed() {
            Ok(x) => if frame_time > x {
                frame_time - x
            } else {
                Duration::new(0, 0)
            },
            Err(_) => Duration::new(0, 0),
        };
        sleep(sleep_time);
    }
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let scale = 5;

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", 160 * scale, 144 * scale)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, 160, 144)
        .unwrap();
    // Create a checker board pattern
    texture
        .with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..144 {
                for x in 0..160 {
                    let offset = y * pitch + x * 3;
                    let v = ((x + y) % 2) as u8 * 0xff;
                    buffer[offset] = v;
                    buffer[offset + 1] = v;
                    buffer[offset + 2] = v;
                }
            }
        })
        .unwrap();

    canvas.clear();
    canvas.copy(&texture, None, None).unwrap();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let cartridge_rom = "../ROMs/tetris.gb";
    let boot_rom = Some("../ROMs/dmg_rom.gb");
    let mut emulator = Emulator::new(boot_rom, cartridge_rom);

    'running: loop {
        let frame_timer = FrameTimer::new(1000000000 / 60);

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        emulator.run_until_vblank();
        texture
            .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                let screen_buffer = emulator.get_screen_buffer();
                for y in 0..144 {
                    for x in 0..160 {
                        let offset = y * pitch + x * 3;
                        // let v = ((x + y) % 2) as u8 * 0xff;
                        let v = screen_buffer[y * 160 + x] * 85;
                        buffer[offset] = v;
                        buffer[offset + 1] = v;
                        buffer[offset + 2] = v;
                    }
                }
            })
            .unwrap();

        canvas.copy(&texture, None, None).unwrap();
        canvas.present();

        frame_timer.sleep_till_end_of_frame();
    }
}
