//! Simulation module skeleton

/// Expose the vehicle module so other modules can use Vehicle, Route, Direction
pub mod vehicle;
pub mod intersection;
mod physics;

pub use vehicle::{Vehicle, Direction, Route};
pub use intersection::Intersection;

/// Placeholder for simulation logic (vehicles, world updates)
pub struct Simulation {
    // add fields here
}

impl Simulation {
    /// Create a new simulation instance
    pub fn new() -> Self {
        Simulation {}
    }
}
