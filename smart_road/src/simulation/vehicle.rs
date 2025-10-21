/// Represents the four cardinal directions a vehicle can come from
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

/// Represents the route a vehicle will take at the intersection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Route {
    Right,
    Straight,
    Left,
}

/// Represents a vehicle in the traffic simulation
/// 
/// Each vehicle has a unique ID, position, velocity, and planned route.
/// The vehicle tracks its distance to the intersection and whether it's
/// currently active in the simulation.
#[derive(Debug, Clone)]
pub struct Vehicle {
    pub id: u32,             
    pub position: (f32, f32),              // Current position in 2D space (x, y coordinates)
    pub velocity: f32,
    pub route: Route,                      // The route this vehicle will take (right, straight, or left)
    pub direction: Direction,              // Direction the vehicle is coming from
    pub distance_to_intersection: f32,     // Distance remaining to the intersection in meters
    pub time_elapsed: f32,                 // in seconds
    pub active: bool,                       // Whether this vehicle is currently active in the simulation
}

impl Vehicle {
    pub fn new(
        id: u32,
        position: (f32, f32),
        velocity: f32,
        route: Route,
        direction: Direction,
        distance_to_intersection: f32,
    ) -> Self {
        Vehicle {
            id,
            position,
            velocity,
            route,
            direction,
            distance_to_intersection,
            time_elapsed: 0.0,
            active: true,
        }
    }

    /// Updates the vehicle's position based on its velocity and the time elapsed
    /// 
    /// Uses basic kinematics: distance = velocity × time
    /// Also updates the distance to intersection and total time elapsed.
    /// 
    /// # Arguments
    /// * `delta_time` - Time elapsed since last update (in seconds)
    pub fn update_position(&mut self, delta_time: f32) {
        // Calculate distance traveled in this time step
        let distance_traveled = self.velocity * delta_time;
        
        // Update time elapsed
        self.time_elapsed += delta_time;
        
        // Update distance to intersection
        self.distance_to_intersection -= distance_traveled;
        
        // Update position based on direction
        match self.direction {
            Direction::North => {
                // Moving upward (positive Y direction)
                self.position.1 += distance_traveled;
            }
            Direction::South => {
                // Moving downward (negative Y direction)
                self.position.1 -= distance_traveled;
            }
            Direction::East => {
                // Moving right (positive X direction)
                self.position.0 += distance_traveled;
            }
            Direction::West => {
                // Moving left (negative X direction)
                self.position.0 -= distance_traveled;
            }
        }
        
        // Deactivate vehicle if it has passed through the intersection
        // (negative distance means it's gone past)
        if self.distance_to_intersection < -50.0 {
            self.active = false;
        }
    }

    /// Checks if this vehicle is too close to another vehicle
    /// 
    /// Calculates the Euclidean distance between two vehicles and compares
    /// it to the safe distance threshold.
    /// 
    /// # Arguments
    /// * `other` - Reference to another vehicle
    /// * `safe_distance` - Minimum safe distance in meters
    /// 
    /// # Returns
    /// `true` if the vehicles are closer than the safe distance, `false` otherwise
    pub fn is_too_close(&self, other: &Vehicle, safe_distance: f32) -> bool {
        // Calculate Euclidean distance between the two vehicles
        let dx = self.position.0 - other.position.0;
        let dy = self.position.1 - other.position.1;
        let distance = (dx * dx + dy * dy).sqrt();
        
        // Check if distance is less than safe distance
        distance < safe_distance
    }

    /// Stops the vehicle (sets velocity to 0)
    pub fn stop(&mut self) {
        self.velocity = 0.0;
    }

    /// Sets the vehicle's velocity
    /// 
    /// # Arguments
    /// * `velocity` - New velocity in m/s
    pub fn set_velocity(&mut self, velocity: f32) {
        self.velocity = velocity;
    }

    /// Checks if the vehicle is currently stopped
    /// 
    /// # Returns
    /// `true` if velocity is 0, `false` otherwise
    pub fn is_stopped(&self) -> bool {
        self.velocity == 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vehicle_creation() {
        let vehicle = Vehicle::new(
            1,
            (0.0, 0.0),
            10.0,
            Route::Straight,
            Direction::North,
            100.0,
        );

        assert_eq!(vehicle.id, 1);
        assert_eq!(vehicle.position, (0.0, 0.0));
        assert_eq!(vehicle.velocity, 10.0);
        assert_eq!(vehicle.route, Route::Straight);
        assert_eq!(vehicle.direction, Direction::North);
        assert_eq!(vehicle.distance_to_intersection, 100.0);
        assert_eq!(vehicle.time_elapsed, 0.0);
        assert!(vehicle.active);
    }

    #[test]
    fn test_update_position_north() {
        let mut vehicle = Vehicle::new(
            1,
            (0.0, 0.0),
            10.0,
            Route::Straight,
            Direction::North,
            100.0,
        );

        vehicle.update_position(1.0);

        assert_eq!(vehicle.position.1, 10.0); // Y increased
        assert_eq!(vehicle.distance_to_intersection, 90.0);
        assert_eq!(vehicle.time_elapsed, 1.0);
    }

    #[test]
    fn test_update_position_south() {
        let mut vehicle = Vehicle::new(
            1,
            (0.0, 100.0),
            10.0,
            Route::Straight,
            Direction::South,
            100.0,
        );

        vehicle.update_position(2.0);

        assert_eq!(vehicle.position.1, 80.0); // Y decreased by 20
        assert_eq!(vehicle.distance_to_intersection, 80.0);
        assert_eq!(vehicle.time_elapsed, 2.0);
    }

    #[test]
    fn test_update_position_east() {
        let mut vehicle = Vehicle::new(
            1,
            (0.0, 0.0),
            5.0,
            Route::Right,
            Direction::East,
            50.0,
        );

        vehicle.update_position(2.0);

        assert_eq!(vehicle.position.0, 10.0); // X increased by 10
        assert_eq!(vehicle.distance_to_intersection, 40.0);
    }

    #[test]
    fn test_update_position_west() {
        let mut vehicle = Vehicle::new(
            1,
            (100.0, 0.0),
            10.0,
            Route::Left,
            Direction::West,
            100.0,
        );

        vehicle.update_position(1.0);

        assert_eq!(vehicle.position.0, 90.0); // X decreased by 10
        assert_eq!(vehicle.distance_to_intersection, 90.0);
    }

    #[test]
    fn test_vehicle_deactivation() {
        let mut vehicle = Vehicle::new(
            1,
            (0.0, 0.0),
            10.0,
            Route::Straight,
            Direction::North,
            10.0,
        );

        // Move vehicle past the intersection
        vehicle.update_position(10.0); // Should make distance = -90
        
        assert!(!vehicle.active);
    }

    #[test]
    fn test_is_too_close() {
        let vehicle1 = Vehicle::new(
            1,
            (0.0, 0.0),
            10.0,
            Route::Straight,
            Direction::North,
            100.0,
        );

        let vehicle2 = Vehicle::new(
            2,
            (3.0, 4.0), // Distance = 5.0
            10.0,
            Route::Straight,
            Direction::North,
            95.0,
        );

        assert!(vehicle1.is_too_close(&vehicle2, 10.0)); // 5 < 10
        assert!(!vehicle1.is_too_close(&vehicle2, 3.0)); // 5 > 3
    }

    #[test]
    fn test_stop_vehicle() {
        let mut vehicle = Vehicle::new(
            1,
            (0.0, 0.0),
            10.0,
            Route::Straight,
            Direction::North,
            100.0,
        );

        vehicle.stop();

        assert_eq!(vehicle.velocity, 0.0);
        assert!(vehicle.is_stopped());
    }

    #[test]
    fn test_set_velocity() {
        let mut vehicle = Vehicle::new(
            1,
            (0.0, 0.0),
            10.0,
            Route::Straight,
            Direction::North,
            100.0,
        );

        vehicle.set_velocity(20.0);
        assert_eq!(vehicle.velocity, 20.0);

        vehicle.set_velocity(0.0);
        assert!(vehicle.is_stopped());
    }

    #[test]
    fn test_multiple_updates() {
        let mut vehicle = Vehicle::new(
            1,
            (0.0, 0.0),
            10.0,
            Route::Straight,
            Direction::North,
            100.0,
        );

        // Update 5 times with 0.5 second intervals
        for _ in 0..5 {
            vehicle.update_position(0.5);
        }

        assert_eq!(vehicle.position.1, 25.0); // 10 * 2.5 seconds
        assert_eq!(vehicle.distance_to_intersection, 75.0);
        assert_eq!(vehicle.time_elapsed, 2.5);
    }

    #[test]
    fn test_route_variants() {
        let right = Vehicle::new(1, (0.0, 0.0), 10.0, Route::Right, Direction::North, 100.0);
        let straight = Vehicle::new(2, (0.0, 0.0), 10.0, Route::Straight, Direction::North, 100.0);
        let left = Vehicle::new(3, (0.0, 0.0), 10.0, Route::Left, Direction::North, 100.0);

        assert_eq!(right.route, Route::Right);
        assert_eq!(straight.route, Route::Straight);
        assert_eq!(left.route, Route::Left);
    }
}