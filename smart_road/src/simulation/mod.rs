//! Simulation module skeleton
use crate::stats::STATS;  // Import the singleton

/// Expose the vehicle module so other modules can use Vehicle, Route, Direction
pub mod vehicle;
pub mod intersection;
pub(crate) mod physics;

pub use vehicle::{Vehicle, Direction, Route};
pub use intersection::Intersection;


pub struct Simulation {
    pub intersection: Intersection,
    pub total_vehicles: u32,
}

impl Simulation {
    /// Create a new simulation instance
    pub fn new(safe_distance: f32) -> Self {
        Simulation {
            intersection: Intersection::new(safe_distance),
            total_vehicles: 0,
        }
    }

    pub fn add_vehicle(&mut self, vehicle: Vehicle) {
        let direction = vehicle.direction;
        if self.intersection.add_vehicle(direction, vehicle) {
            self.total_vehicles += 1;
        }
    }

    /// Get a reference to the intersection for vehicle counting
    pub fn intersection(&self) -> &Intersection {
        &self.intersection
    }

    pub fn update(&mut self, delta_time: f32) {
        self.intersection.update(delta_time);
        // Update car count from active vehicles in intersection
        let active_count = self.intersection.total_vehicles();
        STATS.lock().unwrap().num_cars = active_count as i32;
    }

    pub fn print_statistics(&self) {
        println!("\n=== Simulation Statistics ===");
        println!("Total vehicles processed: {}", self.total_vehicles);
        println!("Active vehicles: {}", self.intersection.total_vehicles());
        for direction in &[Direction::North, Direction::South, Direction::East, Direction::West] {
            println!("{:?}: {}", direction, self.intersection.vehicles_in_lane(*direction));
        }
    }
}
