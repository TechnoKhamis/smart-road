//! Events module skeleton

pub mod input;

pub use input::InputHandler;
use crate::simulation::Direction;

// Events that can occur in the simulation
#[derive(Debug, Clone)]
pub enum SimulationEvent {
    // Simullation should exit and show stats
    Exit,

    // Vehicle spawned
    VehicleSpawned {
        id: u32,
        direction: crate::simulation::vehicle::Direction,
    },

    // Random generation toggled
    RandomGenerationToggled {
        enabled: bool,
    },
}

/// Placeholder for event handling (keyboard, spawn events)
pub enum Event {
    // Define events here
}
