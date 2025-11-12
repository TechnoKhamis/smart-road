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

    /// Enforces safe distance with advanced speed control
    /// 
    /// This method provides smooth deceleration and acceleration
    /// to prevent harsh stops and maintain traffic flow
    pub fn enforce_safe_distance_advanced(
        &self,
        vehicle: &mut Vehicle,
        leading_vehicle: Option<&Vehicle>,
        target_velocity: f32,
    ) {
        if let Some(leader) = leading_vehicle {
            let distance = ((vehicle.position.0 - leader.position.0).powi(2) + 
                           (vehicle.position.1 - leader.position.1).powi(2)).sqrt();
            
            // Calculate required safe following distance
            let safe_following_distance = self.safe_distance + (vehicle.velocity * 0.8) + 2.0;
            
            if distance < safe_following_distance {
                // Too close - calculate appropriate deceleration
                let _speed_difference = vehicle.velocity - leader.velocity;
                let deceleration_factor = if distance < self.safe_distance {
                    0.0 // Emergency stop
                } else if distance < safe_following_distance * 0.8 {
                    0.3 // Heavy deceleration
                } else {
                    0.6 // Moderate deceleration
                };
                
                let new_velocity = (leader.velocity * deceleration_factor).max(0.0);
                vehicle.set_velocity(new_velocity);
            } else if distance > safe_following_distance * 1.5 {
                // Safe distance - can accelerate towards target
                let acceleration_factor = 0.9;
                let new_velocity = (target_velocity * acceleration_factor).min(target_velocity);
                vehicle.set_velocity(new_velocity);
            } else {
                // Good following distance - match leader's speed
                vehicle.set_velocity(leader.velocity);
            }
        } else {
            // No vehicle ahead - can move at target velocity
            vehicle.set_velocity(target_velocity);
        }
    }

    /// Predictive collision avoidance for intersection approach
    /// 
    /// Predicts where vehicles will be in the future and prevents conflicts
    pub fn predict_and_prevent_collision(
        &self,
        vehicle: &mut Vehicle,
        other_vehicles: &[Vehicle],
        prediction_time: f32,
    ) -> bool {
        // Calculate future position of current vehicle
        let future_pos = self.predict_future_position(vehicle, prediction_time);
        
        for other in other_vehicles {
            if other.id == vehicle.id || !other.active {
                continue;
            }
            
            // Calculate future position of other vehicle
            let other_future_pos = self.predict_future_position(other, prediction_time);
            
            // Check if future positions are too close
            let future_distance = ((future_pos.0 - other_future_pos.0).powi(2) + 
                                  (future_pos.1 - other_future_pos.1).powi(2)).sqrt();
            
            if future_distance < self.safe_distance * 1.5 {
                // Potential collision detected - slow down
                let reduction_factor = future_distance / (self.safe_distance * 2.0);
                let safe_velocity = vehicle.velocity * reduction_factor.min(0.5);
                vehicle.set_velocity(safe_velocity);
                return true;
            }
        }
        
        false
    }

    /// Predicts vehicle position after given time
    fn predict_future_position(&self, vehicle: &Vehicle, time: f32) -> (f32, f32) {
        let distance = vehicle.velocity * time;
        
        match vehicle.direction {
            Direction::North => (vehicle.position.0, vehicle.position.1 + distance),
            Direction::South => (vehicle.position.0, vehicle.position.1 - distance),
            Direction::East => (vehicle.position.0 + distance, vehicle.position.1),
            Direction::West => (vehicle.position.0 - distance, vehicle.position.1),
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