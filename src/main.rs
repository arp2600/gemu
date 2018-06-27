extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::TextureAccess;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("rust-sdl2 demo: Video", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().software().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture(None, TextureAccess::Streaming, 32, 32)
        .unwrap();
    let mut pixels = [0; 32*32*4];
    for i in 0..32 {
        pixels[i*32*4 + 0] = 0xff;
        pixels[i*32*4 + 1] = 0xff;
        pixels[i*32*4 + 2] = 0xff;
        pixels[i*32*4 + 3] = 0xff;
    }
    texture.update(None, &pixels, 32*4).unwrap();

    canvas.copy(&texture, None, None).expect("Render failed");
    canvas.present();

    'mainloop: loop {
        for event in sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Option::Some(Keycode::Escape),
                    ..
                } => break 'mainloop,
                _ => {}
            }
        }
    }
}
