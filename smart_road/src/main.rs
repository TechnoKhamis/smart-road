use sdl2::event::Event;
use sdl2::keyboard::Keycode;

mod simulation;
mod events;

use events::InputHandler;
use simulation::Simulation;
use std::time::Duration;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("Traffic Intersection", 800, 600)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let mut event_pump = sdl_context.event_pump()?;

    // Create simulation and input handler
    let mut simulation = Simulation::new(10.0); // 10m safe distance
    let mut input_handler = InputHandler::new(500, 100.0); // 500ms cooldown, 100m spawn distance

    'running: loop {
        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    // Show statistics and exit
                    simulation.print_statistics();
                    break 'running;
                }
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    // Handle vehicle spawning
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
        simulation.update(0.016); // ~60 FPS

        // Render
        canvas.clear();
        // Add your rendering code here
        canvas.present();

        std::thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}
