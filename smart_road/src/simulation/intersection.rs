use std::collections::HashMap;
use super::vehicle::{Vehicle, Direction, Route, INTERSECTION_HALF_WIDTH};
use super::physics::{Physics, velocities};

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
        }
    }

    /// Adds a vehicle to the intersection
    pub fn add_vehicle(&mut self, direction: Direction, vehicle: Vehicle) -> bool {
        if let Some(lane) = self.lanes.get_mut(&direction) {
            lane.push(vehicle);
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
                        
                        // Step 1: Check for vehicle immediately ahead (prevent overlapping)
                        let ahead_distance = {
                            let mut closest_distance = None;
                            
                            for (idx, other) in current_lane.iter().enumerate() {
                                if idx != i && 
                                   other.active && 
                                   other.distance_to_intersection < vehicle.distance_to_intersection {
                                    // Calculate actual distance between vehicles
                                    let dx = vehicle.position.0 - other.position.0;
                                    let dy = vehicle.position.1 - other.position.1;
                                    let distance = (dx * dx + dy * dy).sqrt();
                                    
                                    if closest_distance.is_none() || distance < closest_distance.unwrap() {
                                        closest_distance = Some(distance);
                                    }
                                }
                            }
                            
                            closest_distance
                        };
                        
                        if let Some(dist) = ahead_distance {
                            if dist <= self.safe_distance {
                                vehicle.stop();
                                continue;
                            } else if dist < self.safe_distance * 3.0 {
                                // When close to another vehicle, use SLOW speed instead of stopping
                                vehicle.set_velocity(velocities::SLOW);
                                continue;
                            }
                        }
                        
                        // Step 2: Check intersection safety
                        if vehicle.distance_to_intersection <= self.intersection_entry_distance && 
                           vehicle.distance_to_intersection > 0.0 {
                            let intersection_safe = {
                                let mut safe = true;
                                
                                for (other_dir, other_lane) in &all_lanes_snapshot {
                                    if *other_dir == direction {
                                        continue;
                                    }
                                    
                                    for other in other_lane {
                                        if !other.active {
                                            continue;
                                        }
                                        
                                        if other.distance_to_intersection.abs() < INTERSECTION_HALF_WIDTH * 2.0 {
                                            // Check if paths will cross using simplified logic
                                            let paths_cross = match (vehicle.direction, other.direction, vehicle.route, other.route) {
                                                (Direction::North, Direction::South, Route::Straight, Route::Straight) => true,
                                                (Direction::South, Direction::North, Route::Straight, Route::Straight) => true,
                                                (Direction::East, Direction::West, Route::Straight, Route::Straight) => true,
                                                (Direction::West, Direction::East, Route::Straight, Route::Straight) => true,
                                                (_, _, Route::Left, _) => true,
                                                (_, _, _, Route::Left) => true,
                                                _ => false,
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
                                                
                                                // If both will reach intersection within 3.0 seconds of each other
                                                if (my_time - other_time).abs() < 3.0 && my_time < 8.0 {
                                                    // Check right-of-way using vehicle data
                                                    let has_priority = {
                                                        // Rule 1: Vehicle already in intersection has absolute priority
                                                        if vehicle.distance_to_intersection < 0.0 && other.distance_to_intersection >= 0.0 {
                                                            true
                                                        } else if other.distance_to_intersection < 0.0 && vehicle.distance_to_intersection >= 0.0 {
                                                            false
                                                        } else {
                                                            // Rule 2: Vehicle closer to intersection has priority
                                                            let distance_diff = vehicle.distance_to_intersection - other.distance_to_intersection;
                                                            if distance_diff.abs() > 5.0 {
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
                                                    
                                                    if has_priority {
                                                        // This vehicle has right of way, continue
                                                        continue;
                                                    } else {
                                                        // Other vehicle has right of way, this one yields
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
                                let distance_to_stop_line = vehicle.distance_to_intersection - INTERSECTION_HALF_WIDTH;
                                
                                if distance_to_stop_line <= 20.0 {
                                    // Instead of gradual deceleration, use SLOW speed when unsafe
                                    vehicle.set_velocity(velocities::SLOW);
                                    if distance_to_stop_line <= 5.0 {
                                        // Only stop when very close to intersection
                                        vehicle.stop();
                                    }
                                    continue;
                                }
                            }
                        }
                        
                        // Step 3: Determine appropriate speed (discrete speeds only)
                        let target_speed = {
                            // Determine base speed based on traffic conditions
                            let mut vehicles_nearby: usize = 0;
                            for (idx, other) in current_lane.iter().enumerate() {
                                if idx != i && other.active {
                                    let distance_diff = (vehicle.distance_to_intersection - other.distance_to_intersection).abs();
                                    if distance_diff < 50.0_f32 {
                                        vehicles_nearby += 1;
                                    }
                                }
                            }
                            
                            // Choose discrete speed based on conditions
                            if vehicle.distance_to_intersection < 15.0_f32 {
                                // Very close to intersection - use slow speed
                                velocities::SLOW
                            } else if vehicles_nearby >= 3 {
                                // Heavy traffic - use slow speed
                                velocities::SLOW  
                            } else if vehicle.distance_to_intersection < 30.0_f32 || vehicles_nearby >= 2 {
                                // Approaching intersection or moderate traffic - use medium speed
                                velocities::MEDIUM
                            } else {
                                // Open road - use fast speed
                                velocities::FAST
                            }
                        };
                        
                        // Apply speed: either stop (0.0) or use target speed (no gradual changes)
                        vehicle.set_velocity(target_speed);
                        
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
}