use core::f32;

use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::ttf::Font;

#[derive(Debug)]
pub struct StatisticsManager {
    pub num_cars: i32,
    pub num_close: i32,
    pub max_velo: f32,
    pub min_velo: f32,
}

impl StatisticsManager {
    pub fn new() -> Self {
        StatisticsManager {
            num_cars: 0,
            num_close: 0,
            max_velo: f32::MIN,
            min_velo: f32::MAX,
        }
    }
    
    pub fn render_stats(
        &self, 
        canvas: &mut Canvas<Window>,
        font: &Font,
        texture_creator: &sdl2::render::TextureCreator<sdl2::video::WindowContext>,
    ) -> Result<(), String> {
        // Semi-transparent dark overlay
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 180));
        canvas.fill_rect(Rect::new(0, 0, 700, 700))?;
        
        // Title "STATISTICS"
        let title_surface = font
            .render("STATISTICS")
            .blended(Color::RGB(255, 0, 0))
            .map_err(|e| e.to_string())?;
        let title_texture = texture_creator
            .create_texture_from_surface(&title_surface)
            .map_err(|e| e.to_string())?;
        let title_target = Rect::new(250, 120, title_surface.width(), title_surface.height());
        canvas.copy(&title_texture, None, Some(title_target))?;
        
        // Stats panel background
        canvas.set_draw_color(Color::RGB(50, 50, 50));
        canvas.fill_rect(Rect::new(150, 200, 400, 400))?;
        
        // Car count text
        let cars_text = format!("Total Cars: {}", self.num_cars);
        self.render_text(canvas, font, texture_creator, &cars_text, 200, 250, Color::RGB(255, 255, 255))?;
        
        // Close calls text
        let close_text = format!("Close Calls: {}", self.num_close);
        self.render_text(canvas, font, texture_creator, &close_text, 200, 310, Color::RGB(255, 165, 0))?;
        
        // Max velocity text
        let max_velo_text = if self.num_cars == 0 || self.max_velo == f32::MIN {
            "Max Velocity: N/A".to_string()
        } else {
            format!("Max Velocity: {:.2} m/s", self.max_velo)
        };
        self.render_text(canvas, font, texture_creator, &max_velo_text, 200, 370, Color::RGB(0, 255, 0))?;
        
        // Min velocity text
        let min_velo_text = if self.num_cars == 0 || self.min_velo == f32::MAX {
            "Min Velocity: N/A".to_string()
        } else {
            format!("Min Velocity: {:.2} m/s", self.min_velo)
        };
        self.render_text(canvas, font, texture_creator, &min_velo_text, 200, 430, Color::RGB(0, 200, 255))?;
        
        Ok(())
    }
    
    fn render_text(
        &self,
        canvas: &mut Canvas<Window>,
        font: &Font,
        texture_creator: &sdl2::render::TextureCreator<sdl2::video::WindowContext>,
        text: &str,
        x: i32,
        y: i32,
        color: Color,
    ) -> Result<(), String> {
        let surface = font
            .render(text)
            .blended(color)
            .map_err(|e| e.to_string())?;
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())?;
        let target = Rect::new(x, y, surface.width(), surface.height());
        canvas.copy(&texture, None, Some(target))?;
        Ok(())
    }
    
    pub fn update_car_count(&mut self, count: i32) {
        self.num_cars = count;
    }
    
    pub fn record_close_call(&mut self) {
        self.num_close += 1;
    }
    
    pub fn record_velocity(&mut self, velocity: f32) {
        self.max_velo = self.max_velo.max(velocity);
        self.min_velo = self.min_velo.min(velocity); 
    }
}