use super::vehicle::{Vehicle, Direction};

/// Predefined velocity constants for the simulation
pub mod velocities {
    /// Slow velocity: 5 m/s (~18 km/h)
    pub const SLOW: f32 = 5.0;

    /// Medium velocity: 10 m/s (~36 km/h)
    pub const MEDIUM: f32 = 10.0;

    /// Fast velocity: 15 m/s (~54 km/h)
    pub const FAST: f32 = 15.0;
}

/// Physics engine for the traffic simulation
#[derive(Debug)]
pub struct Physics {
    pub safe_distance: f32,
    pub boundary_limit: f32,
}

impl Physics {
    /// Creates a new physics engine with specified parameters
    pub fn new(safe_distance: f32, boundary_limit: f32) -> Self {
        Physics {
            safe_distance,
            boundary_limit,
        }
    }

    /// Calculates the time required for a vehicle to travel a given distance
    pub fn calculate_time(&self, distance: f32, velocity: f32) -> Option<f32> {
        if velocity > 0.0 {
            Some(distance / velocity)
        } else {
            None
        }
    }

    /// Checks if a vehicle maintains safe distance from another vehicl
    pub fn is_safe_distance(&self, vehicle1: &Vehicle, vehicle2: &Vehicle) -> bool {
        !vehicle1.is_too_close(vehicle2, self.safe_distance)
    }

    /// Checks if a vehicle has left the simulation boundaries
    pub fn is_out_of_bounds(&self, vehicle: &Vehicle) -> bool {
        // Check if vehicle has passed through intersection and is beyond boundary
        vehicle.distance_to_intersection < -self.boundary_limit
    }

    /// Enforces safe distance by adjusting vehicle velocity
    pub fn enforce_safe_distance(
        &self,
        vehicle: &mut Vehicle,
        leading_vehicle: Option<&Vehicle>,
        target_velocity: f32,
    ) {
        if let Some(leader) = leading_vehicle {
            if !self.is_safe_distance(vehicle, leader) {
                // Too close - stop the vehicle
                vehicle.stop();
            } else {
                // Safe distance maintained - can move at target velocity
                vehicle.set_velocity(target_velocity);
            }
        } else {
            // No vehicle ahead - can move at target velocity
            vehicle.set_velocity(target_velocity);
        }
    }

    /// Gets the recommended velocity based on distance to intersection
    /// Vehicles slow down as they approach the intersection for safety.
    pub fn get_adjusted_velocity(&self, distance_to_intersection: f32, base_velocity: f32) -> f32 {
        if distance_to_intersection < 20.0 {
            // Very close to intersection - reduce to slow speed
            base_velocity * 0.5
        } else if distance_to_intersection < 50.0 {
            // Approaching intersection - reduce to 75% speed
            base_velocity * 0.75
        } else {
            // Far from intersection - maintain base speed
            base_velocity
        }
    }
}