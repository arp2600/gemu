extern crate gb_emu;
extern crate gb_synth;
extern crate sdl2;
mod frame_timer;

use frame_timer::FrameTimer;
use gb_emu::{App, AudioAction, Command, Emulator, JoyPad};
use gb_synth::{AudioServer, SynthController};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Canvas, RenderTarget, Texture};
use sdl2::EventPump;
use std::env;

pub const VRAM_START: usize = 0x8000;
pub const VRAM_END: usize = 0x9fff;
pub const OAM_START: usize = 0xfe00;
pub const OAM_END: usize = 0xfe9f;
pub const IO_START: usize = 0xff00;
pub const IO_END: usize = 0xff7f;

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

enum StopReason {
    Quit,
    DumpVideoMemory,
}

struct Runner<'a, 'b, F: RenderTarget> {
    canvas: Canvas<F>,
    texture: Texture<'a>,
    texture_buffer: [u8; 160 * 144 * 3],
    event_pump: EventPump,
    frame_timer: FrameTimer,
    stop_reason: StopReason,
    frame_count: u64,
    synth: SynthController<'b>,
}

impl<'r, 's, F: RenderTarget> App for Runner<'r, 's, F> {
    fn draw_line(&mut self, line_buffer: &[u8], line_index: u8) {
        let y = usize::from(line_index);
        for (x, v) in line_buffer.iter().enumerate() {
            let offset = y * 160 * 3 + x * 3;
            let v = *v * 85;
            self.texture_buffer[offset] = v;
            self.texture_buffer[offset + 1] = v;
            self.texture_buffer[offset + 2] = v;
        }

        if line_index == 143 {
            self.texture
                .update(None, &self.texture_buffer, 160 * 3)
                .unwrap();
            self.canvas.copy(&self.texture, None, None).unwrap();
            self.canvas.present();
        }
    }

    fn update(&mut self, joypad: &mut JoyPad) -> Command {
        self.frame_count += 1;
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    self.stop_reason = StopReason::Quit;
                    return Command::Stop;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::M),
                    ..
                } => {
                    self.stop_reason = StopReason::DumpVideoMemory;
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

        self.frame_timer.sleep_then_update();

        Command::Continue
    }

    fn update_audio(&mut self, action: AudioAction, _cpu_time: f64) {
        match action {
            AudioAction::SetFrequency(chan, freq) => {
                self.synth.set_freq(chan, freq);
            }
            AudioAction::SetAmplitude(chan, amp) => {
                self.synth.set_amp(chan, amp);
            }
            AudioAction::SetPulseWidth(chan, width) => {
                self.synth.set_pulse_width(chan, width);
            }
            AudioAction::RestartSound(chan) => {
                self.synth.reset_channel(chan);
            }
            AudioAction::SetWavetable(index, value) => {
                self.synth.set_wavetable(index, value);
            }
            AudioAction::SetEnvelope(chan, amp, duration) => {
                self.synth.set_envelope(chan, amp, duration);
            }
        }
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

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let scale = 5;
    let window = video_subsystem
        .window("rust-sdl2 demo: Video", 160 * scale, 144 * scale)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut audio_server = AudioServer::new();
    let synth = audio_server.get_synth_controller();

    let mut emulator = {
        let boot_rom = None;
        Emulator::new(boot_rom, &cartridge_rom)
    };

    let canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut runner = {
        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB24, 160, 144)
            .unwrap();
        let mut event_pump = sdl_context.event_pump().unwrap();
        let mut frame_timer = FrameTimer::new(59.73);
        Runner {
            canvas,
            texture,
            texture_buffer: [0; 160 * 144 * 3],
            event_pump,
            frame_timer,
            stop_reason: StopReason::Quit,
            frame_count: 0,
            synth: synth,
        }
    };

    loop {
        emulator.run(&mut runner);
        match runner.stop_reason {
            StopReason::Quit => break,
            StopReason::DumpVideoMemory => {
                print!("VRAM:");
                for i in VRAM_START..=VRAM_END {
                    let v = emulator.read_memory(i as u16);
                    print!(" {:02x}", v);
                }
                println!("");

                print!("OAM:");
                for i in OAM_START..=OAM_END {
                    let v = emulator.read_memory(i as u16);
                    print!(" {:02x}", v);
                }
                println!("");

                print!("IO:");
                for i in IO_START..=IO_END {
                    let v = emulator.read_memory(i as u16);
                    print!(" {:02x}", v);
                }
                println!("");
            }
        }
    }

    println!("Ran for {} frames", runner.frame_count);
}
