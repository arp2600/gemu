extern crate gb_emu;
extern crate sdl2;
mod frame_timer;

use frame_timer::FrameTimer;
use gb_emu::{Command, Emulator};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;

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

    let mut emulator = {
        let cartridge_rom = "../ROMs/tetris.gb";
        let boot_rom = Some("../ROMs/dmg_rom.gb");
        Emulator::new(boot_rom, cartridge_rom)
    };

    let draw_fn = {
        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB24, 160, 144)
            .unwrap();

        let mut texture_buffer = [0; 160 * 144 * 3];

        move |line: &[u8], line_index: u8| {
            let y = usize::from(line_index);
            for (x, v) in line.iter().enumerate() {
                let offset = y * 160 * 3 + x * 3;
                let v = *v * 85;
                texture_buffer[offset] = v;
                texture_buffer[offset + 1] = v;
                texture_buffer[offset + 2] = v;
            }

            if line_index == 143 {
                texture.update(None, &texture_buffer, 160 * 3).unwrap();
                canvas.copy(&texture, None, None).unwrap();
                canvas.present();
            }
        }
    };

    let update_fn = {
        let mut event_pump = sdl_context.event_pump().unwrap();
        let mut frame_timer = FrameTimer::new(59.73);

        move || {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => return Command::Stop,
                    _ => {}
                }
            }

            frame_timer.sleep_then_update();

            Command::Continue
        }
    };

    emulator.run(draw_fn, update_fn);
}
