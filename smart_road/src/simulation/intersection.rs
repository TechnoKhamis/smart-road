use std::collections::HashMap;
use super::vehicle::{Vehicle, Direction, Route};
use super::physics::Physics;


#[derive(Debug)]
pub struct Intersection {
    /// Stores vehicles in each lane, organized by direction
    /// Each direction (North, South, East, West) has its own queue of vehicles
    pub lanes: HashMap<Direction, Vec<Vehicle>>,
    
    /// Minimum safe distance between vehicles (in meters)
    pub safe_distance: f32,
    pub physics: Physics,
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
        }
    }

    /// Checks if a vehicle can safely enter the intersection
    /// 
    /// A vehicle can enter if:
    /// 1. There are no vehicles in its lane, OR
    /// 2. All vehicles in its lane are at a safe distance
    pub fn can_enter(&self, vehicle: &Vehicle) -> bool {
        // Get the vehicles in the same lane
        if let Some(lane_vehicles) = self.lanes.get(&vehicle.direction) {
            // If lane is empty, vehicle can enter
            if lane_vehicles.is_empty() {
                return true;
            }
            
            // Check if vehicle is too close to any vehicle in the same lane
            for other in lane_vehicles {
                if vehicle.is_too_close(other, self.safe_distance) {
                    return false;
                }
            }
            
            // Also check for potential collisions with vehicles from other directions
            // that might cross paths (simplified check)
            for (direction, vehicles) in &self.lanes {
                // Skip same direction
                if *direction == vehicle.direction {
                    continue;
                }
                
                // Check for crossing path conflicts
                for other in vehicles {
                    // If vehicles are crossing paths, check if they're too close
                    if self.paths_cross(vehicle, other) && vehicle.is_too_close(other, self.safe_distance * 1.5) {
                        return false;
                    }
                }
            }
        }
        
        true
    }

    /// Checks if two vehicles' paths will cross at the intersection
    fn paths_cross(&self, v1: &Vehicle, v2: &Vehicle) -> bool {
        // Opposite directions going straight cross each other
        match (v1.direction, v2.direction) {
            (Direction::North, Direction::South) | (Direction::South, Direction::North) => {
                matches!(v1.route, Route::Straight) && matches!(v2.route, Route::Straight)
            }
            (Direction::East, Direction::West) | (Direction::West, Direction::East) => {
                matches!(v1.route, Route::Straight) && matches!(v2.route, Route::Straight)
            }
            // Left turns cross most paths
            _ => matches!(v1.route, Route::Left) || matches!(v2.route, Route::Left)
        }
    }

    /// Adds a vehicle to the intersection
    /// 
    /// The vehicle is added to the appropriate lane based on its direction.
    /// The vehicle is only added if it can safely enter.
    pub fn add_vehicle(&mut self, direction: Direction, vehicle: Vehicle) -> bool {
        // Check if vehicle can safely enter
        if !self.can_enter(&vehicle) {
            return false;
        }
        
        // Add vehicle to the appropriate lane
        if let Some(lane) = self.lanes.get_mut(&direction) {
            lane.push(vehicle);
            return true;
        }
        
        false
    }

    /// Updates all vehicles in the intersection
    /// 
    /// This method:
    /// 1. Updates the position of each active vehicle
    /// 2. Removes vehicles that have completed their journey through the intersection
    pub fn update(&mut self, delta_time: f32) {
        // Update each lane
        for lane in self.lanes.values_mut() {
            // Update positions of all vehicles
            for vehicle in lane.iter_mut() {
                if vehicle.active {
                    vehicle.update_position(delta_time);
                    // Use physics to check boundaries
                    if self.physics.is_out_of_bounds(vehicle) {
                        vehicle.active = false;
                    }
                }
            }
            
            // Remove inactive vehicles (those that have passed through)
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