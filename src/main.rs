extern crate gb_emu;
extern crate sdl2;
mod frame_timer;

use frame_timer::FrameTimer;
use gb_emu::{Command, Emulator, JoyPad};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use std::env;

fn set_button(joypad: &mut JoyPad, keycode: Keycode, state: bool) {
    match keycode {
        Keycode::U => joypad.set_a(state),
        Keycode::I => joypad.set_b(state),
        Keycode::O => joypad.set_select(state),
        Keycode::P => joypad.set_start(state),
        Keycode::W => joypad.set_up(state),
        Keycode::A => joypad.set_left(state),
        Keycode::S => joypad.set_down(state),
        Keycode::D => joypad.set_right(state),
        _ => (),
    }
}

pub fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("error: add rom file as argument");
        eprintln!("usage `cargo run -- <rom_file>`");
        return;
    }

    let cartridge_rom = &args[1];
    // println!("args: {:?}", args);
    // return;

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
        // let cartridge_rom = "../ROMs/tetris.gb";
        // let cartridge_rom = "../ROMs/dr_mario.gb";
        // let cartridge_rom = "../ROMs/rocky_and_bullwinkle.gb";
        // let cartridge_rom = "../ROMs/super_mario_land_1.1.gb";
        // let cartridge_rom = "../ROMs/ultima_runes_of_virtue.gb";
        // let cartridge_rom = "../ROMs/pokemon_blue.gb";
        // let cartridge_rom = "gb_emu/gb-test-roms/cpu_instrs/cpu_instrs.gb";
        // let boot_rom = Some("../ROMs/dmg_rom.gb");
        let boot_rom = None;
        Emulator::new(boot_rom, &cartridge_rom)
    };
    // emulator.set_tracing(true);

    let draw_fn = {
        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB24, 160, 144)
            .unwrap();

        let mut texture_buffer = [0; 160 * 144 * 3];

        move |line: &[u8], line_index: u8| {
            let y = usize::from(line_index);
            for (x, v) in line.iter().enumerate() {
                let offset = y * 160 * 3 + x * 3;
                match *v {
                    0...3 => {
                        let v = *v * 85;
                        // let v = *v * 15;
                        texture_buffer[offset] = v;
                        texture_buffer[offset + 1] = v;
                        texture_buffer[offset + 2] = v;
                    }
                    // 4...7 => {
                    //     let v = ((*v - 4) * 25) + 150;
                    //     texture_buffer[offset] = v;
                    //     texture_buffer[offset + 1] = 0;
                    //     texture_buffer[offset + 2] = 0;
                    // }
                    // 8...11 => {
                    //     let v = ((*v - 8) * 25) + 150;
                    //     texture_buffer[offset] = 0;
                    //     texture_buffer[offset + 1] = 0;
                    //     texture_buffer[offset + 2] = v;
                    // }
                    _ => panic!("error: bad color value!"),
                }
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

        move |joypad: &mut JoyPad| {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => {
                        return Command::Stop;
                    }
                    Event::KeyDown {
                        keycode: Some(x), ..
                    } => set_button(joypad, x, true),
                    Event::KeyUp {
                        keycode: Some(x), ..
                    } => set_button(joypad, x, false),
                    _ => {}
                }
            }

            frame_timer.sleep_then_update();

            Command::Continue
        }
    };

    emulator.run(draw_fn, update_fn);
}
