//! Simulation module skeleton
use std::rc::Rc;
use std::cell::RefCell;
use crate::StatisticsManager;  // Add this import

/// Expose the vehicle module so other modules can use Vehicle, Route, Direction
pub mod vehicle;
pub mod intersection;
pub(crate) mod physics;

pub use vehicle::{Vehicle, Direction, Route};
pub use intersection::Intersection;

/// Placeholder for simulation logic (vehicles, world updates)
pub struct Simulation {
    pub intersection: Intersection,
    pub total_vehicles: u32,
    stats: Rc<RefCell<StatisticsManager>>,  // Add this field

}

impl Simulation {
    /// Create a new simulation instance
    pub fn new(safe_distance: f32,stats: Rc<RefCell<StatisticsManager>> ) -> Self {
        Simulation {
            intersection: Intersection::new(safe_distance),
            total_vehicles: 0,
            stats:stats,
        }
    }

    pub fn add_vehicle(&mut self, vehicle: Vehicle) {
        let direction = vehicle.direction;
        if self.intersection.add_vehicle(direction, vehicle,Rc::clone(&self.stats)) {
            self.total_vehicles += 1;
            self.stats.borrow_mut().update_car_count(self.total_vehicles as i32);
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.intersection.update(delta_time, Rc::clone(&self.stats));
        // Update car count from active vehicles in intersection
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
