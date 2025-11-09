use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use crate::simulation::vehicle::{Vehicle, Direction, Route};
use super::assets::AssetManager;

/// Lane configuration constants
pub mod lanes {
    /// Width of each lane in meters
    pub const LANE_WIDTH: f32 = 3.5;

    /// Number of lanes in each direction
    pub const LANES_PER_DIRECTION: i32 = 3;

    /// Total road width on each side (3 lanes * 3.5m)
    pub const ROAD_WIDTH: f32 = LANE_WIDTH * 3.0;

    /// Intersection size (center area)
    pub const INTERSECTION_SIZE: f32 = ROAD_WIDTH * 2.0;
    
    /// Width of the median separator in meters
    pub const SEPARATOR_WIDTH: f32 = 0.5;
}

/// Handles animation and rendering of vehicles
pub struct AnimationManager {
    /// Size of vehicle sprite in meters
    vehicle_size: f32,
}

impl AnimationManager {
    pub fn new(vehicle_size: f32) -> Self {
        AnimationManager { vehicle_size }
    }

    /// Renders all vehicles in the simulation
    pub fn render_vehicles(
        &self,
        canvas: &mut Canvas<Window>,
        vehicles: &Vec<&Vehicle>,
        assets: &AssetManager,
    ) -> Result<(), String> {
        for vehicle in vehicles {
            if vehicle.active {
                self.render_vehicle(canvas, vehicle, assets)?;
            }
        }
        Ok(())
    }

    /// Renders a single vehicle
    fn render_vehicle(
        &self,
        canvas: &mut Canvas<Window>,
        vehicle: &Vehicle,
        assets: &AssetManager,
    ) -> Result<(), String> {
        // Get the appropriate texture
        let texture = assets.get_vehicle_texture(vehicle.direction)
            .ok_or("Vehicle texture not found")?;

        // Calculate adjusted position based on lane and route
        let (world_x, world_y) = self.calculate_vehicle_position(vehicle);

        // Convert to screen coordinates
        let (screen_x, screen_y) = assets.world_to_screen(world_x, world_y);

        // Calculate sprite size in pixels
        let sprite_size = (self.vehicle_size * assets.scale) as u32;

        // Create destination rectangle (centered on vehicle position)
        let dest_rect = Rect::new(
            screen_x - (sprite_size / 2) as i32,
            screen_y - (sprite_size / 2) as i32,
            sprite_size,
            sprite_size,
        );

        // Draw the vehicle
        canvas.copy(texture, None, Some(dest_rect))?;

        Ok(())
    }

    /// Calculates the actual world position of a vehicle based on its lane
    fn calculate_vehicle_position(&self, vehicle: &Vehicle) -> (f32, f32) {
        let base_x = vehicle.position.0;
        let base_y = vehicle.position.1;
        let lane_offset = self.get_lane_offset(vehicle.route);

        // Physics now transfers offset at the moment of turning.
        // Renderer applies offset only based on current direction.
        match vehicle.direction {
            Direction::North => (base_x + lane_offset, base_y),
            Direction::South => (base_x - lane_offset, base_y),
            Direction::East  => (base_x, base_y - lane_offset),
            Direction::West  => (base_x, base_y + lane_offset),
        }
    }

    /// Gets the lane offset based on the route
    fn get_lane_offset(&self, route: Route) -> f32 {
        match route {
            // was: Right => 0.5, Left => 2.5 (inverted)
            Route::Right => lanes::LANE_WIDTH * 2.5,     // outer/rightmost lane
            Route::Straight => lanes::LANE_WIDTH * 1.5,  // middle lane
            Route::Left => lanes::LANE_WIDTH * 0.5,      // inner/leftmost lane
        }
    }

    /// Draws the intersection layout
    pub fn draw_intersection(
        &self,
        canvas: &mut Canvas<Window>,
        assets: &AssetManager,
    ) -> Result<(), String> {
        use sdl2::pixels::Color;

        // Road color (dark gray)
        canvas.set_draw_color(Color::RGB(50, 50, 50));

        let half_width = (lanes::ROAD_WIDTH * assets.scale) as i32;

        // Draw horizontal road (East-West)
        let h_road = Rect::new(
            0,
            assets.center_y - half_width,
            assets.center_x as u32 * 2,
            (half_width * 2) as u32,
        );
        canvas.fill_rect(h_road)?;

        // Draw vertical road (North-South)
        let v_road = Rect::new(
            assets.center_x - half_width,
            0,
            (half_width * 2) as u32,
            assets.center_y as u32 * 2,
        );
        canvas.fill_rect(v_road)?;

        // Draw white separators (median dividers) - only outside intersection
        self.draw_separators(canvas, assets)?;

        // Draw lane markings (only outside intersection area)
        self.draw_lane_markings(canvas, assets)?;

        Ok(())
    }

    /// Draws white median separators, excluding the intersection area
    fn draw_separators(
        &self,
        canvas: &mut Canvas<Window>,
        assets: &AssetManager,
    ) -> Result<(), String> {
        use sdl2::pixels::Color;

        canvas.set_draw_color(Color::RGB(255, 255, 255)); // White color

        let separator_width = (lanes::SEPARATOR_WIDTH * assets.scale) as u32;
        let separator_half = (separator_width / 2) as i32;
        let half_width = (lanes::ROAD_WIDTH * assets.scale) as i32;

        // Horizontal separator (East-West road) - split into two parts
        // Left side (from left edge to intersection left edge)
        let h_separator_left = Rect::new(
            0,
            assets.center_y - separator_half,
            (assets.center_x - half_width) as u32,
            separator_width,
        );
        canvas.fill_rect(h_separator_left)?;

        // Right side (from intersection right edge to right edge)
        let h_separator_right = Rect::new(
            assets.center_x + half_width,
            assets.center_y - separator_half,
            (assets.center_x - half_width) as u32,
            separator_width,
        );
        canvas.fill_rect(h_separator_right)?;

        // Vertical separator (North-South road) - split into two parts
        // Top side (from top edge to intersection top edge)
        let v_separator_top = Rect::new(
            assets.center_x - separator_half,
            0,
            separator_width,
            (assets.center_y - half_width) as u32,
        );
        canvas.fill_rect(v_separator_top)?;

        // Bottom side (from intersection bottom edge to bottom edge)
        let v_separator_bottom = Rect::new(
            assets.center_x - separator_half,
            assets.center_y + half_width,
            separator_width,
            (assets.center_y - half_width) as u32,
        );
        canvas.fill_rect(v_separator_bottom)?;

        Ok(())
    }

    /// Draws lane markings (dashed white lines), excluding the intersection area
    fn draw_lane_markings(
        &self,
        canvas: &mut Canvas<Window>,
        assets: &AssetManager,
    ) -> Result<(), String> {
        let lane_width_px = (lanes::LANE_WIDTH * assets.scale) as i32;
        let dash_length = 20;
        let dash_gap = 15;
        
        // Calculate intersection boundaries
        let half_width = (lanes::ROAD_WIDTH * assets.scale) as i32;

        // Horizontal lane markings (for North-South road)
        for i in 1..3 {
            let x = assets.center_x + (i * lane_width_px);
            // Draw from top to intersection top edge
            self.draw_dashed_line(canvas, x, 0, x, assets.center_y - half_width, dash_length, dash_gap)?;
            // Draw from intersection bottom edge to bottom
            self.draw_dashed_line(canvas, x, assets.center_y + half_width, x, assets.center_y * 2, dash_length, dash_gap)?;

            let x_neg = assets.center_x - (i * lane_width_px);
            // Draw from top to intersection top edge
            self.draw_dashed_line(canvas, x_neg, 0, x_neg, assets.center_y - half_width, dash_length, dash_gap)?;
            // Draw from intersection bottom edge to bottom
            self.draw_dashed_line(canvas, x_neg, assets.center_y + half_width, x_neg, assets.center_y * 2, dash_length, dash_gap)?;
        }

        // Vertical lane markings (for East-West road)
        for i in 1..3 {
            let y = assets.center_y + (i * lane_width_px);
            // Draw from left to intersection left edge
            self.draw_dashed_line(canvas, 0, y, assets.center_x - half_width, y, dash_length, dash_gap)?;
            // Draw from intersection right edge to right
            self.draw_dashed_line(canvas, assets.center_x + half_width, y, assets.center_x * 2, y, dash_length, dash_gap)?;

            let y_neg = assets.center_y - (i * lane_width_px);
            // Draw from left to intersection left edge
            self.draw_dashed_line(canvas, 0, y_neg, assets.center_x - half_width, y_neg, dash_length, dash_gap)?;
            // Draw from intersection right edge to right
            self.draw_dashed_line(canvas, assets.center_x + half_width, y_neg, assets.center_x * 2, y_neg, dash_length, dash_gap)?;
        }

        Ok(())
    }

    /// Draws a dashed line
    fn draw_dashed_line(
        &self,
        canvas: &mut Canvas<Window>,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        dash_length: i32,
        dash_gap: i32,
    ) -> Result<(), String> {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let total_length = ((dx * dx + dy * dy) as f32).sqrt() as i32;
        let dash_cycle = dash_length + dash_gap;

        for i in (0..total_length).step_by(dash_cycle as usize) {
            let t1 = i as f32 / total_length as f32;
            let t2 = ((i + dash_length).min(total_length) as f32) / total_length as f32;

            let start_x = x1 + (dx as f32 * t1) as i32;
            let start_y = y1 + (dy as f32 * t1) as i32;
            let end_x = x1 + (dx as f32 * t2) as i32;
            let end_y = y1 + (dy as f32 * t2) as i32;

            canvas.draw_line((start_x, start_y), (end_x, end_y))?;
        }

        Ok(())
    }
}
