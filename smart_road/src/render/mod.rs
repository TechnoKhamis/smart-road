mod assets;
mod animation;

pub use assets::AssetManager;
pub use animation::{AnimationManager};

use sdl2::render::Canvas;
use sdl2::video::Window;
use crate::simulation::Simulation;

pub struct Renderer<'a> {
    pub assets: AssetManager<'a>,
    pub animation: AnimationManager,
}

impl<'a> Renderer<'a> {
    pub fn new(assets: AssetManager<'a>) -> Self {
        Renderer {
            assets,
            animation: AnimationManager::new(5.5), // Increased from 4.0 to 5.5 meters for bigger cars
        }
    }

    pub fn render(
        &self,
        canvas: &mut Canvas<Window>,
        simulation: &Simulation,
    ) -> Result<(), String> {
        use sdl2::pixels::Color;

        // Clear screen (grass/background)
        canvas.set_draw_color(Color::RGB(34, 139, 34));
        canvas.clear();

        // Draw intersection
        self.animation.draw_intersection(canvas, &self.assets)?;

        // Draw all vehicles
        let all_vehicles: Vec<_> = simulation.intersection.lanes
            .values()
            .flat_map(|lane| lane.iter())
            .collect();

        self.animation.render_vehicles(canvas, &all_vehicles, &self.assets)?;

        Ok(())
    }
}
