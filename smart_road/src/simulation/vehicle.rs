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
    pub prev_direction: Direction,
}

const LANE_WIDTH: f32 = 3.5;
const LANES_PER_DIRECTION: f32 = 3.0;
pub const INTERSECTION_HALF_WIDTH: f32 = LANE_WIDTH * LANES_PER_DIRECTION; // 10.5m

const TURN_SHIFT: f32 = -2.0;

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
            match route {
                Route::Right    => LANE_WIDTH * 2.5,
                Route::Straight => LANE_WIDTH * 1.5,
                Route::Left     => LANE_WIDTH * 0.5,
            }
        }

        // Right-turn: turn at intersection.
        if self.route == Route::Right && !self.has_turned {
            let turn_edge = INTERSECTION_HALF_WIDTH + TURN_SHIFT;
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
    pub fn set_velocity(&mut self, velocity: f32) {
        self.velocity = velocity;
    }

    /// Checks if the vehicle is currently stopped
    pub fn is_stopped(&self) -> bool {
        self.velocity == 0.0
    }
}