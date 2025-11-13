use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

mod simulation;
mod events;
mod render;
mod stats;

use events::InputHandler;
use simulation::Simulation;
use render::{AssetManager, Renderer};
use stats::StatisticsManager;
use std::time::Duration;

use std::rc::Rc;
use std::cell::RefCell;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(sdl2::image::InitFlag::PNG)?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let window = video_subsystem
        .window("Traffic Intersection", 700, 700)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();

    let assets = AssetManager::new(&texture_creator, 700, 700, 10.0)?;
    let renderer = Renderer::new(assets);
    
    // Load font
    let font = ttf_context.load_font("assets/fonts/OpenSans-Bold.ttf", 24)?;

    let mut event_pump = sdl_context.event_pump()?;

        let stats_manager = Rc::new(RefCell::new(StatisticsManager::new()));


    let mut simulation = Simulation::new(25.0, Rc::clone(&stats_manager));
    let mut input_handler = InputHandler::new(500, 100.0);
    
    let mut show_stats = false;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Some(Keycode::Q), .. } => {
                    simulation.print_statistics();
                    break 'running;
                }
                Event::KeyDown { keycode: Some(Keycode::ESCAPE), .. } => {
                    show_stats = !show_stats;
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

        if let Some(vehicle) = input_handler.update_random_generation(800) {
            simulation.add_vehicle(vehicle);
        }

        simulation.update(0.016);
        //stats_manager.update_car_count(simulation.vehicles.len() as i32);

        renderer.render(&mut canvas, &simulation)?;
        
        if show_stats {
            stats_manager.borrow().render_stats(&mut canvas, &font, &texture_creator)?;
        }
        
        canvas.present();

        std::thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}