use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

mod simulation;
mod events;
mod render;

use events::InputHandler;
use simulation::Simulation;
use render::{AssetManager, Renderer};
use std::time::Duration;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(sdl2::image::InitFlag::PNG)?;

    let window = video_subsystem
        .window("Traffic Intersection", 700, 700)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();

    // Create asset manager and renderer
    // Adjusted scale to 10.0 pixels per meter for balanced view in square window
    let assets = AssetManager::new(&texture_creator, 700, 700, 10.0)?;
    let renderer = Renderer::new(assets);

    let mut event_pump = sdl_context.event_pump()?;

    // Create simulation and input handler
    let mut simulation = Simulation::new(25.0);
    let mut input_handler = InputHandler::new(500, 100.0);

    'running: loop {
        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    simulation.print_statistics();
                    break 'running;
                }
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    let vehicles = input_handler.handle_keypress(keycode);
                    for vehicle in vehicles {
                        simulation.add_vehicle(vehicle);
                    }
                }
                _ => {}
            }
        }

        // Update random vehicle generation
        if let Some(vehicle) = input_handler.update_random_generation(800) {
            simulation.add_vehicle(vehicle);
        }

        // Update simulation
        simulation.update(0.016);

        // Render
        renderer.render(&mut canvas, &simulation)?;
        canvas.present();

        std::thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}
