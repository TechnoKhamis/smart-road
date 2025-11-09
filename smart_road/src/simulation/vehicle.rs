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
    pub active: bool,
    pub time_elapsed: f32,

    // <-- new field to track completed right-turn
    pub has_turned: bool,
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
            active: true,
            time_elapsed: 0.0,
            // initialize new field
            has_turned: false,
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
        let distance_traveled = self.velocity * delta_time;

        fn direction_right_of(dir: Direction) -> Direction {
            match dir {
                Direction::North => Direction::East,
                Direction::East => Direction::South,
                Direction::South => Direction::West,
                Direction::West => Direction::North,
            }
        }

        if self.route == Route::Right && !self.has_turned {
            // Remaining distance to intersection center before this tick
            let remaining = self.distance_to_intersection;

            if remaining <= distance_traveled {
                // Move up to center
                let to_center = remaining.max(0.0);
                match self.direction {
                    Direction::North => self.position.1 += to_center,
                    Direction::South => self.position.1 -= to_center,
                    Direction::East  => self.position.0 += to_center,
                    Direction::West  => self.position.0 -= to_center,
                }

                // Turn (always) at center
                self.direction = direction_right_of(self.direction);
                self.has_turned = true;

                // Move leftover distance along new direction
                let after_turn = distance_traveled - to_center;
                match self.direction {
                    Direction::North => self.position.1 += after_turn,
                    Direction::South => self.position.1 -= after_turn,
                    Direction::East  => self.position.0 += after_turn,
                    Direction::West  => self.position.0 -= after_turn,
                }

                // Distance to original center now reduced by full traveled amount
                self.distance_to_intersection -= distance_traveled;
                self.time_elapsed += delta_time;
            } else {
                // Not at center yet: advance straight toward center
                match self.direction {
                    Direction::North => self.position.1 += distance_traveled,
                    Direction::South => self.position.1 -= distance_traveled,
                    Direction::East  => self.position.0 += distance_traveled,
                    Direction::West  => self.position.0 -= distance_traveled,
                }
                self.distance_to_intersection -= distance_traveled;
                self.time_elapsed += delta_time;
            }
        } else {
            // Straight or left routes, or already turned right
            match self.direction {
                Direction::North => self.position.1 += distance_traveled,
                Direction::South => self.position.1 -= distance_traveled,
                Direction::East  => self.position.0 += distance_traveled,
                Direction::West  => self.position.0 -= distance_traveled,
            }
            self.distance_to_intersection -= distance_traveled;
            self.time_elapsed += delta_time;
        }

        // Deactivate vehicle if it has passed through the intersection far enough
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

    #[test]
    fn test_right_lane_turns() {
        // Approaching from South going North, in right lane (Route::Right)
        // Start 5m away, velocity high enough to reach & turn in one tick
        let mut v = Vehicle::new(
            10,
            (0.0, -5.0),          // 5m south of center (since moving North we increase y)
            10.0,                 // 10 m/s
            Route::Right,
            Direction::North,
            5.0,                  // distance_to_intersection
        );

        v.update_position(0.6);   // travels 6m (passes center, should turn)

        assert!(v.has_turned, "Right-route vehicle must mark has_turned");
        assert_eq!(v.direction, Direction::East, "Vehicle should have turned right (North -> East)");
        // After moving: 5m to center, 1m after turn along East => x should be ~1.0
        assert!((v.position.0 - 1.0).abs() < 0.001, "Post-turn X displacement incorrect");
    }
}