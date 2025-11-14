#[derive(Clone, Copy)]
enum LanePos { Left, Middle, Right }

use sdl2::keyboard::Keycode;
use rand::Rng;
use std::time::{Duration, Instant};
use crate::simulation::vehicle::{Direction, Route, Vehicle};
use crate::simulation::physics::velocities;

/// Handles keyboard input for vehicle generation
pub struct InputHandler {
    /// Minimum time between vehicle spawns for the same key (prevents spam)
    spawn_cooldown: Duration,

    /// Last spawn time for each direction
    last_spawn_north: Option<Instant>,
    last_spawn_south: Option<Instant>,
    last_spawn_east: Option<Instant>,
    last_spawn_west: Option<Instant>,

    /// Last time a random vehicle was spawned (for R key)
    last_random_spawn: Option<Instant>,

    /// Whether continuous random generation is enabled (R key)
    pub random_generation_enabled: bool,

    /// Counter for vehicle IDs
    next_vehicle_id: u32,

    /// Initial distance from intersection for spawned vehicles
    spawn_distance: f32,
}

impl InputHandler {
    pub fn new(spawn_cooldown_ms: u64, spawn_distance: f32) -> Self {
        InputHandler {
            spawn_cooldown: Duration::from_millis(spawn_cooldown_ms),
            last_spawn_north: None,
            last_spawn_south: None,
            last_spawn_east: None,
            last_spawn_west: None,
            last_random_spawn: None,
            random_generation_enabled: false,
            next_vehicle_id: 1,
            spawn_distance,
        }
    }

    /// Handles a keypress event and returns vehicles to spawn (if any)
    pub fn handle_keypress(&mut self, keycode: Keycode) -> Vec<Vehicle> {
        match keycode {
            Keycode::Up => self.try_spawn_vehicle(Direction::North),
            Keycode::Down => self.try_spawn_vehicle(Direction::South),
            Keycode::Right => self.try_spawn_vehicle(Direction::East),
            Keycode::Left => self.try_spawn_vehicle(Direction::West),
            Keycode::R => {
                self.random_generation_enabled = !self.random_generation_enabled;
                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    /// Attempts to spawn a vehicle from a specific direction
    /// Checks cooldown to prevent vehicles spawning on top of each other
    fn try_spawn_vehicle(&mut self, direction: Direction) -> Vec<Vehicle> {
        let now = Instant::now();

        let last_spawn = match direction {
            Direction::North => &mut self.last_spawn_north,
            Direction::South => &mut self.last_spawn_south,
            Direction::East => &mut self.last_spawn_east,
            Direction::West => &mut self.last_spawn_west,
        };

        // Check if cooldown has elapsed
        if let Some(last) = last_spawn {
            if now.duration_since(*last) < self.spawn_cooldown {
                return Vec::new(); // Still in cooldown
            }
        }

        // Update last spawn time
        *last_spawn = Some(now);

        // Create and return the vehicle
        vec![self.create_vehicle(direction)]
    }

    /// Updates random vehicle generation (called each frame)
    /// Now limits to max 2 vehicles per direction
    pub fn update_random_generation(&mut self, random_spawn_rate_ms: u64, intersection: &crate::simulation::Intersection) -> Option<Vehicle> {
        if !self.random_generation_enabled {
            return None;
        }

        let now = Instant::now();
        let spawn_interval = Duration::from_millis(random_spawn_rate_ms);

        // Check if it's time to spawn a random vehicle
        if let Some(last) = self.last_random_spawn {
            if now.duration_since(last) < spawn_interval {
                return None;
            }
        }

        self.last_random_spawn = Some(now);

        // Generate a random direction
        let direction = Self::random_direction();
        
        // Check if this direction already has too many vehicles (max 2)
        if intersection.vehicles_in_lane(direction) >= 2 {
            return None; // Skip spawning if direction has 2 or more vehicles
        }
        
        Some(self.create_vehicle(direction))
    }

    /// Creates a new vehicle with lane-based route selection
    fn create_vehicle(&mut self, direction: Direction) -> Vehicle {
        let id = self.next_vehicle_id;
        self.next_vehicle_id += 1;

        // Pick a lane at spawn, then map lane -> route
        let lane = Self::random_lane();
        let route = Self::route_from_lane(lane);

        let velocity = Self::random_velocity();
        let position = Self::get_spawn_position(direction, self.spawn_distance);

        Vehicle::new(
            id,
            position,
            velocity,
            route,
            direction,
            self.spawn_distance,
        )
    }

    // -- lane/route helpers --


    fn route_from_lane(lane: LanePos) -> Route {
        match lane {
            LanePos::Right => Route::Right,
            LanePos::Middle => Route::Straight,
            LanePos::Left => Route::Left,
        }
    }

    fn random_lane() -> LanePos {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..3) {
            0 => LanePos::Right,
            1 => LanePos::Middle,
            _ => LanePos::Left,
        }
    }

    fn random_velocity() -> f32 {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..3) {
            0 => velocities::SLOW,
            1 => velocities::MEDIUM,
            _ => velocities::FAST,
        }
    }

    fn random_direction() -> Direction {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..4) {
            0 => Direction::North,
            1 => Direction::South,
            2 => Direction::East,
            _ => Direction::West,
        }
    }

    /// Calculates spawn position based on direction
    fn get_spawn_position(direction: Direction, distance: f32) -> (f32, f32) {
        match direction {
            Direction::North => (0.0, -distance),  // Spawns south of center, moving north
            Direction::South => (0.0, distance),   // Spawns north of center, moving south
            Direction::East => (-distance, 0.0),   // Spawns west of center, moving east
            Direction::West => (distance, 0.0),    // Spawns east of center, moving west
        }
    }

    /// Resets the input handler state
    pub fn reset(&mut self) {
        self.last_spawn_north = None;
        self.last_spawn_south = None;
        self.last_spawn_east = None;
        self.last_spawn_west = None;
        self.last_random_spawn = None;
        self.random_generation_enabled = false;
    }
}