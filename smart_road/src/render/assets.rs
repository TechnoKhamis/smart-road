use sdl2::image::LoadTexture;
use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;
use std::collections::HashMap;
use crate::simulation::vehicle::Direction;

/// Manages loading and storing of graphical assets
pub struct AssetManager<'a> {
    /// Vehicle textures for each direction
    vehicle_textures: HashMap<Direction, Texture<'a>>,

    /// World scale: pixels per meter
    pub scale: f32,

    /// Screen center coordinates
    pub center_x: i32,
    pub center_y: i32,
}

impl<'a> AssetManager<'a> {
    /// Creates a new asset manager and loads all textures
    pub fn new(
        texture_creator: &'a TextureCreator<WindowContext>,
        window_width: u32,
        window_height: u32,
        scale: f32,
    ) -> Result<Self, String> {
        let mut vehicle_textures = HashMap::new();

        // Load vehicle sprites for each direction
        let north_texture = texture_creator.load_texture("assets/vehicle/NORTH.png")?;
        let south_texture = texture_creator.load_texture("assets/vehicle/SOUTH.png")?;
        let east_texture = texture_creator.load_texture("assets/vehicle/EAST.png")?;
        let west_texture = texture_creator.load_texture("assets/vehicle/WEST.png")?;

        vehicle_textures.insert(Direction::North, north_texture);
        vehicle_textures.insert(Direction::South, south_texture);
        vehicle_textures.insert(Direction::East, east_texture);
        vehicle_textures.insert(Direction::West, west_texture);

        Ok(AssetManager {
            vehicle_textures,
            scale,
            center_x: (window_width / 2) as i32,
            center_y: (window_height / 2) as i32,
        })
    }

    /// Gets the vehicle texture for a specific direction
    pub fn get_vehicle_texture(&self, direction: Direction) -> Option<&Texture> {
        self.vehicle_textures.get(&direction)
    }

    /// Converts world coordinates (meters) to screen coordinates (pixels)
    pub fn world_to_screen(&self, world_x: f32, world_y: f32) -> (i32, i32) {
        let screen_x = self.center_x + (world_x * self.scale) as i32;
        let screen_y = self.center_y - (world_y * self.scale) as i32; // Invert Y axis
        (screen_x, screen_y)
    }
}
