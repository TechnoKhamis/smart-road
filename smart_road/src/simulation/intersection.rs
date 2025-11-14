use std::collections::{HashMap, HashSet};
use super::vehicle::{Vehicle, Direction, Route, INTERSECTION_HALF_WIDTH};
use super::physics::{Physics, velocities};
use crate::stats::STATS;  // Import the singleton

/// Traffic intersection with zero-collision management
#[derive(Debug)]
pub struct Intersection {
    /// Vehicle lanes organized by direction
    pub lanes: HashMap<Direction, Vec<Vehicle>>,
    /// Minimum safe distance between vehicles (meters)
    pub safe_distance: f32,
    /// Physics engine for calculations
    pub physics: Physics,
    /// Distance from intersection center where vehicles must evaluate safety
    pub intersection_entry_distance: f32,
    /// Track vehicle pairs that have already had a close call recorded
    recorded_close_calls: HashSet<(u32, u32)>,
}

impl Intersection {
    /// Creates a new intersection with the specified safe distance
    pub fn new(safe_distance: f32) -> Self {
        let mut lanes = HashMap::new();

        // Initialize empty vehicle queues for each direction
        lanes.insert(Direction::North, Vec::new());
        lanes.insert(Direction::South, Vec::new());
        lanes.insert(Direction::East, Vec::new());
        lanes.insert(Direction::West, Vec::new());

        Intersection {
            lanes,
            safe_distance,
            physics: Physics::new(safe_distance, 100.0),
            intersection_entry_distance: INTERSECTION_HALF_WIDTH + 10.0, // 10m before intersection
            recorded_close_calls: HashSet::new(),
        }
    }

    /// Adds a vehicle to the intersection
    pub fn add_vehicle(&mut self, direction: Direction, vehicle: Vehicle) -> bool {
        if let Some(lane) = self.lanes.get_mut(&direction) {
            lane.push(vehicle);
            STATS.lock().unwrap().update_car_count();
            return true;
        }
        false
    }

    /// Main update loop with zero-collision logic
    pub fn update(&mut self, delta_time: f32) {
        // Create a snapshot of all lanes ONCE before any mutations
        let all_lanes_snapshot: HashMap<Direction, Vec<Vehicle>> = self.lanes.iter()
            .map(|(dir, vehicles)| (*dir, vehicles.clone()))
            .collect();

        // Get set of all active vehicle IDs for cleanup
        let active_ids: HashSet<u32> = all_lanes_snapshot.values()
            .flat_map(|lane| lane.iter())
            .filter(|v| v.active)
            .map(|v| v.id)
            .collect();

        // Clean up recorded close calls for inactive vehicles
        self.recorded_close_calls.retain(|(id1, id2)| {
            active_ids.contains(id1) && active_ids.contains(id2)
        });

        // Track close calls before processing vehicles
        self.detect_close_calls(&all_lanes_snapshot);

        // Process each lane separately to avoid borrowing issues
        let directions = [Direction::North, Direction::South, Direction::East, Direction::West];

        for &direction in &directions {
            if let Some(lane) = self.lanes.get_mut(&direction) {
                // Sort by distance to intersection (farthest first for processing)
                lane.sort_by(|a, b| a.distance_to_intersection.partial_cmp(&b.distance_to_intersection).unwrap());
                
                // Process each vehicle in this lane
                for i in 0..lane.len() {
                    if let Some(vehicle) = lane.get_mut(i) {
                        if !vehicle.active {
                            continue;
                        }

                        // Apply zero-collision logic inline to avoid borrowing issues
                        let current_lane = all_lanes_snapshot.get(&direction).unwrap();
                        
                        // Step 1: Dynamic speed control based on proximity to other vehicles
                        let (closest_ahead_distance, closest_any_distance) = {
                            let mut closest_ahead = None;
                            let mut closest_any = None;
                            
                            // Check vehicles in same lane (ahead)
                            for (idx, other) in current_lane.iter().enumerate() {
                                if idx != i && other.active {
                                    let dx = vehicle.position.0 - other.position.0;
                                    let dy = vehicle.position.1 - other.position.1;
                                    let distance = (dx * dx + dy * dy).sqrt();
                                    
                                    // Vehicle ahead in same lane
                                    if other.distance_to_intersection < vehicle.distance_to_intersection {
                                        if closest_ahead.is_none() || distance < closest_ahead.unwrap() {
                                            closest_ahead = Some(distance);
                                        }
                                    }
                                    
                                    // Any nearby vehicle
                                    if closest_any.is_none() || distance < closest_any.unwrap() {
                                        closest_any = Some(distance);
                                    }
                                }
                            }
                            
                            // Check vehicles from other lanes that might be close
                            for (other_dir, other_lane) in &all_lanes_snapshot {
                                if *other_dir != direction {
                                    for other in other_lane {
                                        if other.active {
                                            let dx = vehicle.position.0 - other.position.0;
                                            let dy = vehicle.position.1 - other.position.1;
                                            let distance = (dx * dx + dy * dy).sqrt();
                                            
                                            if distance < 30.0 { // Only consider nearby vehicles
                                                if closest_any.is_none() || distance < closest_any.unwrap() {
                                                    closest_any = Some(distance);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            
                            (closest_ahead, closest_any)
                        };
                        
                        // Dynamic speed adjustment based on proximity (RIGHT TURNS NEVER STOP)
                        let proximity_speed = if vehicle.route == Route::Right {
                            // Right-turning vehicles never stop - minimum SLOW speed
                            if let Some(dist) = closest_ahead_distance {
                                if dist < self.safe_distance * 1.2 {
                                    velocities::SLOW // Very close - but never stop
                                } else if dist < self.safe_distance * 2.5 {
                                    velocities::MEDIUM // Moderately close
                                } else {
                                    velocities::FAST // Safe distance
                                }
                            } else {
                                velocities::FAST // No vehicles ahead
                            }
                        } else {
                            // Non-right-turn vehicles - more aggressive adaptive control
                            if let Some(dist) = closest_ahead_distance {
                                if dist <= self.safe_distance * 0.8 {
                                    0.0 // Stop only if extremely close
                                } else if dist < self.safe_distance * 1.5 {
                                    velocities::SLOW // Closer threshold - more responsive
                                } else if dist < self.safe_distance * 2.5 {
                                    velocities::MEDIUM // Reduced threshold
                                } else {
                                    velocities::FAST // Safe distance - full speed
                                }
                            } else if let Some(dist) = closest_any_distance {
                                if dist < self.safe_distance * 1.0 {
                                    velocities::SLOW // Tighter threshold
                                } else if dist < self.safe_distance * 2.0 {
                                    velocities::MEDIUM // More responsive
                                } else {
                                    velocities::FAST // Safe from all vehicles
                                }
                            } else {
                                velocities::FAST // No vehicles nearby
                            }
                        };
                        
                        // Apply proximity-based speed (RIGHT TURNS NEVER STOP)
                        if proximity_speed == 0.0 && vehicle.route != Route::Right {
                            vehicle.stop();
                            continue;
                        }
                        
                        // Step 2: Check intersection safety
                        // Step 2: Smart intersection safety for multiple vehicles
                        if vehicle.distance_to_intersection <= self.intersection_entry_distance && 
                           vehicle.distance_to_intersection > 0.0 {
                            
                            // Special case: Right-turning vehicles can almost always proceed
                            if vehicle.route == Route::Right {
                                // Only check for vehicles extremely close in the same direction
                                let mut right_turn_safe = true;
                                for (idx, other) in current_lane.iter().enumerate() {
                                    if idx != i && other.active {
                                        let distance = ((vehicle.position.0 - other.position.0).powi(2) + 
                                                       (vehicle.position.1 - other.position.1).powi(2)).sqrt();
                                        if distance < self.safe_distance * 0.8 {
                                            right_turn_safe = false;
                                            break;
                                        }
                                    }
                                }
                                
                                if !right_turn_safe {
                                    vehicle.set_velocity(velocities::SLOW);
                                    continue;
                                }
                            } else {
                                // Enhanced intersection safety for non-right-turn vehicles
                                let intersection_safe = {
                                    let mut safe = true;
                                    let mut vehicles_in_intersection = 0;
                                    
                                    for (other_dir, other_lane) in &all_lanes_snapshot {
                                        if *other_dir == direction {
                                            continue;
                                        }
                                        
                                        for other in other_lane {
                                            if !other.active {
                                                continue;
                                            }
                                            
                                            // Count vehicles currently in intersection
                                            if other.distance_to_intersection < 0.0 && 
                                               other.distance_to_intersection > -INTERSECTION_HALF_WIDTH * 2.0 {
                                                vehicles_in_intersection += 1;
                                            }
                                            
                                            // More precise conflict detection
                                            if other.distance_to_intersection.abs() < INTERSECTION_HALF_WIDTH * 3.0 {
                                                let paths_cross = match (vehicle.route, other.route) {
                                                    (Route::Right, _) => false,  // Right-turning vehicle never conflicts
                                                    (_, Route::Right) => false,  // Other right-turning vehicle never conflicts
                                                    (Route::Left, _) => true,    // Left turns conflict with everything
                                                    (_, Route::Left) => true,    // Anything conflicts with left turns
                                                    // Straight-to-straight conflicts only on opposing lanes
                                                    (Route::Straight, Route::Straight) => match (vehicle.direction, other.direction) {
                                                        (Direction::North, Direction::South) => true,
                                                        (Direction::South, Direction::North) => true,
                                                        (Direction::East, Direction::West) => true,
                                                        (Direction::West, Direction::East) => true,
                                                        _ => false,
                                                    }
                                                };
                                                
                                                if paths_cross {
                                                    let my_time = if vehicle.velocity > 0.0 {
                                                        vehicle.distance_to_intersection / vehicle.velocity
                                                    } else {
                                                        f32::INFINITY
                                                    };
                                                    
                                                    let other_time = if other.velocity > 0.0 {
                                                        other.distance_to_intersection.abs() / other.velocity
                                                    } else {
                                                        f32::INFINITY
                                                    };
                                                    
                                                    // More aggressive timing for faster flow - allow up to 6 vehicles in intersection
                                                    let time_buffer = if vehicles_in_intersection < 3 { 1.8 } else if vehicles_in_intersection < 6 { 2.2 } else { 3.5 };
                                                    
                                                    if (my_time - other_time).abs() < time_buffer && my_time < 8.0 {
                                                        let has_priority = {
                                                            // Rule 1: Vehicle already in intersection has absolute priority
                                                            if vehicle.distance_to_intersection < 0.0 && other.distance_to_intersection >= 0.0 {
                                                                true
                                                            } else if other.distance_to_intersection < 0.0 && vehicle.distance_to_intersection >= 0.0 {
                                                                false
                                                            } else {
                                                                // Rule 2: Vehicle closer to intersection has priority (more generous)
                                                                let distance_diff = vehicle.distance_to_intersection - other.distance_to_intersection;
                                                                if distance_diff.abs() > 8.0 {
                                                                    vehicle.distance_to_intersection < other.distance_to_intersection
                                                                } else {
                                                                    // Rule 3: Route priority (Straight > Right > Left)
                                                                    let vehicle_route_priority = match vehicle.route {
                                                                        Route::Straight => 3,
                                                                        Route::Right => 2,
                                                                        Route::Left => 1,
                                                                    };
                                                                    let other_route_priority = match other.route {
                                                                        Route::Straight => 3,
                                                                        Route::Right => 2,
                                                                        Route::Left => 1,
                                                                    };
                                                                    
                                                                    if vehicle_route_priority != other_route_priority {
                                                                        vehicle_route_priority > other_route_priority
                                                                    } else {
                                                                        // Rule 4: Direction priority
                                                                        let vehicle_priority = match vehicle.direction {
                                                                            Direction::North => 4,
                                                                            Direction::East => 3,
                                                                            Direction::South => 2,
                                                                            Direction::West => 1,
                                                                        };
                                                                        let other_priority = match other.direction {
                                                                            Direction::North => 4,
                                                                            Direction::East => 3,
                                                                            Direction::South => 2,
                                                                            Direction::West => 1,
                                                                        };
                                                                        
                                                                        if vehicle_priority != other_priority {
                                                                            vehicle_priority > other_priority
                                                                        } else {
                                                                            // Final tie-breaker: lower ID has priority
                                                                            vehicle.id < other.id
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        
                                                        if !has_priority {
                                                            safe = false;
                                                            break;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        
                                        if !safe {
                                            break;
                                        }
                                    }
                                    
                                    safe
                                };
                                
                                if !intersection_safe {
                                    // Faster response with shorter slowdown zone
                                    let distance_to_stop_line = vehicle.distance_to_intersection - INTERSECTION_HALF_WIDTH;
                                    
                                    if distance_to_stop_line <= 15.0 {
                                        vehicle.set_velocity(velocities::SLOW);
                                        if distance_to_stop_line <= 2.0 {
                                            vehicle.stop();
                                        }
                                        continue;
                                    }
                                }
                            }
                        }
                        
                        // Step 3: Apply the most conservative speed from proximity and context
                        let final_speed = {
                            // More aggressive context-based speed with shorter thresholds
                            let context_speed = if vehicle.distance_to_intersection < 8.0_f32 {
                                velocities::SLOW  // Very close to intersection
                            } else if vehicle.distance_to_intersection < 25.0_f32 {
                                velocities::MEDIUM  // Approaching intersection (reduced from 40m)
                            } else {
                                velocities::FAST  // Far from intersection
                            };
                            
                            // Use the slower of proximity speed and context speed for safety
                            if proximity_speed <= context_speed {
                                proximity_speed
                            } else {
                                context_speed
                            }
                        };
                        
                        // Apply speed (only using the three predefined speeds)
                        vehicle.set_velocity(final_speed);
                        
                        // Update position
                        if vehicle.velocity > 0.0 {
                            vehicle.update_position(delta_time);
                        }
                        
                        // Check if vehicle is out of bounds
                        if self.physics.is_out_of_bounds(vehicle) {
                            vehicle.active = false;
                        }
                    }
               
                }
            }
        }
        
        // Remove inactive vehicles
        for lane in self.lanes.values_mut() {
            lane.retain(|v| v.active);
        }
    }


    /// Gets the total number of vehicles currently in the intersection
    pub fn total_vehicles(&self) -> usize {
        self.lanes.values().map(|lane| lane.len()).sum()
    }

    /// Gets the number of vehicles in a specific lane
    pub fn vehicles_in_lane(&self, direction: Direction) -> usize {
        self.lanes.get(&direction).map_or(0, |lane| lane.len())
    }

    /// Detects close calls between vehicles
    ///
    /// A close call is when two active vehicles pass within the safe distance threshold.
    /// The algorithm tracks when vehicles enter and exit the "danger zone" to count each
    /// close call only once per encounter.
    fn detect_close_calls(&mut self, all_lanes: &HashMap<Direction, Vec<Vehicle>>) {
        let mut checked_pairs = HashSet::new();
        let mut currently_close_pairs = HashSet::new();

        // Check all pairs of vehicles from different lanes
        for (_dir1, lane1) in all_lanes.iter() {
            for vehicle1 in lane1.iter() {
                if !vehicle1.active {
                    continue;
                }

                for (_dir2, lane2) in all_lanes.iter() {
                    for vehicle2 in lane2.iter() {
                        if !vehicle2.active || vehicle1.id == vehicle2.id {
                            continue;
                        }

                        // Create a unique pair identifier (order doesn't matter)
                        let pair = if vehicle1.id < vehicle2.id {
                            (vehicle1.id, vehicle2.id)
                        } else {
                            (vehicle2.id, vehicle1.id)
                        };

                        // Skip if we already checked this pair in this frame
                        if checked_pairs.contains(&pair) {
                            continue;
                        }
                        checked_pairs.insert(pair);

                        // Calculate distance between vehicles
                        let dx = vehicle1.position.0 - vehicle2.position.0;
                        let dy = vehicle1.position.1 - vehicle2.position.1;
                        let distance = (dx * dx + dy * dy).sqrt();

                        // Check if vehicles are currently too close
                        if distance < self.safe_distance {
                            currently_close_pairs.insert(pair);

                            // Only record as a close call if this is a NEW violation
                            // (not already being tracked)
                            if !self.recorded_close_calls.contains(&pair) {
                                // Close call detected: record it
                                self.recorded_close_calls.insert(pair);
                                STATS.lock().unwrap().record_close_call();
                            }
                        }
                    }
                }
            }
        }

        // Reset tracking for pairs that are no longer close
        // This allows the same pair to be counted again if they separate and come close later
        self.recorded_close_calls.retain(|pair| {
            currently_close_pairs.contains(pair)
        });
    }
}
