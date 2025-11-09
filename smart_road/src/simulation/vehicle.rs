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
    pub position: (f32, f32),
    pub velocity: f32,
    pub route: Route,
    pub direction: Direction,
    pub distance_to_intersection: f32,
    pub active: bool,
    pub time_elapsed: f32,
    pub has_turned: bool,

    // NEW: original direction before current (used to keep lane offset continuity)
    pub prev_direction: Direction,
}

const LANE_WIDTH: f32 = 3.5;
const LANES_PER_DIRECTION: f32 = 3.0;
pub const INTERSECTION_HALF_WIDTH: f32 = LANE_WIDTH * LANES_PER_DIRECTION; // 10.5m

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
            has_turned: false,
            prev_direction: direction, // initialize
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

        #[inline]
        fn direction_right_of(dir: Direction) -> Direction {
            match dir {
                Direction::North => Direction::East,
                Direction::East  => Direction::South,
                Direction::South => Direction::West,
                Direction::West  => Direction::North,
            }
        }

        #[inline]
        fn move_along(pos: &mut (f32, f32), dir: Direction, dist: f32) {
            match dir {
                Direction::North => pos.1 += dist,
                Direction::South => pos.1 -= dist,
                Direction::East  => pos.0 += dist,
                Direction::West  => pos.0 -= dist,
            }
        }

        #[inline]
        fn lane_offset_for_route(route: Route) -> f32 {
            // Must match renderer: Right=2.5, Straight=1.5, Left=0.5 lanes
            match route {
                Route::Right    => LANE_WIDTH * 2.5,
                Route::Straight => LANE_WIDTH * 1.5,
                Route::Left     => LANE_WIDTH * 0.5,
            }
        }

        // Right-turn: turn at intersection entry edge and transfer lateral offset into base position.
        if self.route == Route::Right && !self.has_turned {
            let turn_edge = INTERSECTION_HALF_WIDTH;
            let remaining_to_center = self.distance_to_intersection;
            let to_entry_edge = (remaining_to_center - turn_edge).max(0.0);

            if to_entry_edge <= distance_traveled + f32::EPSILON {
                // 1) move up to the entry edge
                let to_edge = to_entry_edge.min(distance_traveled);
                if to_edge > 0.0 {
                    move_along(&mut self.position, self.direction, to_edge);
                }

                // 2) compute lateral offset vectors for old and new directions and adjust base position
                let offset = lane_offset_for_route(self.route);

                // offset vector as renderer expects: (dx, dy)
                fn offset_vec(dir: Direction, offset: f32) -> (f32, f32) {
                    match dir {
                        Direction::North => ( offset,  0.0),
                        Direction::South => (-offset,  0.0),
                        Direction::East  => ( 0.0, -offset),
                        Direction::West  => ( 0.0,  offset),
                    }
                }

                let old_dir = self.direction;
                let new_dir = direction_right_of(old_dir);

                let old_off = offset_vec(old_dir, offset);
                let new_off = offset_vec(new_dir, offset);

                // base_after = base_before + old_off - new_off  (preserves world_before == world_after)
                self.position.0 += old_off.0 - new_off.0;
                self.position.1 += old_off.1 - new_off.1;

                // 3) turn right
                self.direction = new_dir;
                self.has_turned = true;
                self.prev_direction = old_dir;

                // 4) move remaining distance after the turn along new direction
                let after_turn = (distance_traveled - to_edge).max(0.0);
                if after_turn > 0.0 {
                    move_along(&mut self.position, self.direction, after_turn);
                }

                // bookkeeping
                self.distance_to_intersection -= distance_traveled;
                self.time_elapsed += delta_time;

                if self.distance_to_intersection < -50.0 {
                    self.active = false;
                }
                return;
            }
        }

        // Default straight movement or post-turn movement
        move_along(&mut self.position, self.direction, distance_traveled);
        self.distance_to_intersection -= distance_traveled;
        self.time_elapsed += delta_time;

        if self.distance_to_intersection < -50.0 {
            self.active = false;
        }

        // Keep prev_direction in sync when no turn happened this tick
        if !self.has_turned {
            self.prev_direction = self.direction;
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