//! Simulation module skeleton

/// Expose the vehicle module so other modules can use Vehicle, Route, Direction
pub mod vehicle;
pub use vehicle::{Vehicle, Direction, Route};

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
