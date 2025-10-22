use super::vehicle::{Vehicle, Direction};

/// Predefined velocity constants for the simulation
/// These represent different vehicle speeds in meters per second (m/s)
pub mod velocities {
    /// Slow velocity: 5 m/s (~18 km/h)
    pub const SLOW: f32 = 5.0;

    /// Medium velocity: 10 m/s (~36 km/h)
    pub const MEDIUM: f32 = 10.0;

    /// Fast velocity: 15 m/s (~54 km/h)
    pub const FAST: f32 = 15.0;
}

/// Physics engine for the traffic simulation
///
/// Handles time calculations, safety distance enforcement, and boundary checking
#[derive(Debug)]
pub struct Physics {
    /// Minimum safe distance between vehicles (in meters)
    pub safe_distance: f32,

    /// Boundary limits for removing vehicles (in meters from intersection center)
    pub boundary_limit: f32,
}

impl Physics {
    /// Creates a new physics engine with specified parameters
    ///
    /// # Arguments
    /// * `safe_distance` - Minimum safe distance between vehicles in meters
    /// * `boundary_limit` - Distance from center at which vehicles are removed
    pub fn new(safe_distance: f32, boundary_limit: f32) -> Self {
        Physics {
            safe_distance,
            boundary_limit,
        }
    }

    /// Calculates the time required for a vehicle to travel a given distance
    ///
    /// Uses the formula: time = distance / velocity
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
    ///
    /// A vehicle is considered out of bounds if it has traveled beyond
    /// the boundary limit past the intersection center.
    pub fn is_out_of_bounds(&self, vehicle: &Vehicle) -> bool {
        // Check if vehicle has passed through intersection and is beyond boundary
        vehicle.distance_to_intersection < -self.boundary_limit
    }

    /// Enforces safe distance by adjusting vehicle velocity
    ///
    /// If vehicles are too close, the following vehicle is stopped.
    /// Otherwise, the vehicle can maintain its target velocity.
    ///
    /// # Arguments
    /// * `vehicle` - The vehicle to adjust
    /// * `leading_vehicle` - The vehicle ahead (if any)
    /// * `target_velocity` - Desired velocity for the vehicle
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

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::vehicle::Route;

    #[test]
    fn test_velocity_constants() {
        assert_eq!(velocities::SLOW, 5.0);
        assert_eq!(velocities::MEDIUM, 10.0);
        assert_eq!(velocities::FAST, 15.0);
    }

    #[test]
    fn test_physics_creation() {
        let physics = Physics::new(10.0, 50.0);
        assert_eq!(physics.safe_distance, 10.0);
        assert_eq!(physics.boundary_limit, 50.0);
    }

    #[test]
    fn test_calculate_time() {
        let physics = Physics::new(10.0, 50.0);

        // distance = 100m, velocity = 10 m/s -> time = 10s
        assert_eq!(physics.calculate_time(100.0, 10.0), Some(10.0));

        // distance = 50m, velocity = 5 m/s -> time = 10s
        assert_eq!(physics.calculate_time(50.0, 5.0), Some(10.0));

        // Zero velocity should return None
        assert_eq!(physics.calculate_time(100.0, 0.0), None);
    }

    #[test]
    fn test_is_safe_distance() {
        let physics = Physics::new(10.0, 50.0);

        let vehicle1 = Vehicle::new(
            1,
            (0.0, 0.0),
            10.0,
            Route::Straight,
            Direction::North,
            100.0,
        );

        // Vehicle 20 meters away - safe
        let vehicle2 = Vehicle::new(
            2,
            (0.0, 20.0),
            10.0,
            Route::Straight,
            Direction::North,
            80.0,
        );

        assert!(physics.is_safe_distance(&vehicle1, &vehicle2));

        // Vehicle 5 meters away - too close
        let vehicle3 = Vehicle::new(
            3,
            (0.0, 5.0),
            10.0,
            Route::Straight,
            Direction::North,
            95.0,
        );

        assert!(!physics.is_safe_distance(&vehicle1, &vehicle3));
    }

    #[test]
    fn test_is_out_of_bounds() {
        let physics = Physics::new(10.0, 50.0);

        let mut vehicle = Vehicle::new(
            1,
            (0.0, 0.0),
            10.0,
            Route::Straight,
            Direction::North,
            100.0,
        );

        // Vehicle approaching intersection - not out of bounds
        assert!(!physics.is_out_of_bounds(&vehicle));

        // Vehicle past intersection but within boundary
        vehicle.distance_to_intersection = -30.0;
        assert!(!physics.is_out_of_bounds(&vehicle));

        // Vehicle beyond boundary limit
        vehicle.distance_to_intersection = -60.0;
        assert!(physics.is_out_of_bounds(&vehicle));
    }

    #[test]
    fn test_enforce_safe_distance_no_leader() {
        let physics = Physics::new(10.0, 50.0);

        let mut vehicle = Vehicle::new(
            1,
            (0.0, 0.0),
            5.0,
            Route::Straight,
            Direction::North,
            100.0,
        );

        physics.enforce_safe_distance(&mut vehicle, None, velocities::MEDIUM);
        assert_eq!(vehicle.velocity, velocities::MEDIUM);
    }

    #[test]
    fn test_enforce_safe_distance_with_safe_leader() {
        let physics = Physics::new(10.0, 50.0);

        let mut vehicle = Vehicle::new(
            1,
            (0.0, 0.0),
            5.0,
            Route::Straight,
            Direction::North,
            100.0,
        );

        let leader = Vehicle::new(
            2,
            (0.0, 20.0),
            10.0,
            Route::Straight,
            Direction::North,
            80.0,
        );

        physics.enforce_safe_distance(&mut vehicle, Some(&leader), velocities::FAST);
        assert_eq!(vehicle.velocity, velocities::FAST);
    }

    #[test]
    fn test_enforce_safe_distance_too_close() {
        let physics = Physics::new(10.0, 50.0);

        let mut vehicle = Vehicle::new(
            1,
            (0.0, 0.0),
            10.0,
            Route::Straight,
            Direction::North,
            100.0,
        );

        let leader = Vehicle::new(
            2,
            (0.0, 5.0),
            10.0,
            Route::Straight,
            Direction::North,
            95.0,
        );

        physics.enforce_safe_distance(&mut vehicle, Some(&leader), velocities::MEDIUM);
        assert_eq!(vehicle.velocity, 0.0);
        assert!(vehicle.is_stopped());
    }

    #[test]
    fn test_get_adjusted_velocity_far() {
        let physics = Physics::new(10.0, 50.0);

        let adjusted = physics.get_adjusted_velocity(100.0, velocities::FAST);
        assert_eq!(adjusted, velocities::FAST);
    }

    #[test]
    fn test_get_adjusted_velocity_approaching() {
        let physics = Physics::new(10.0, 50.0);

        let adjusted = physics.get_adjusted_velocity(30.0, velocities::FAST);
        assert_eq!(adjusted, velocities::FAST * 0.75);
    }

    #[test]
    fn test_get_adjusted_velocity_very_close() {
        let physics = Physics::new(10.0, 50.0);

        let adjusted = physics.get_adjusted_velocity(10.0, velocities::MEDIUM);
        assert_eq!(adjusted, velocities::MEDIUM * 0.5);
    }
}
