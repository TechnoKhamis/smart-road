mod simulation;
mod render;
mod events;
mod stats;

use sdl2::init;
use sdl2::pixels::Color;
use std::time::Duration;

fn main() {
    let sdl_context = init().expect("Failed to initialize SDL2");
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Smart Road", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    // create a Canvas from the window so we can draw
    let mut canvas = window.into_canvas().accelerated().build().unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            use sdl2::event::Event;
            use sdl2::keyboard::Keycode;

            match event {
                Event::Quit {..}
                | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                _ => {}
            }
        }

        // set the background color to blue (R,G,B)
        canvas.set_draw_color(Color::RGB(0, 0, 255));
        canvas.clear();
        canvas.present();

        // simple frame cap to avoid 100% CPU
        std::thread::sleep(Duration::from_millis(16));
    }
}
