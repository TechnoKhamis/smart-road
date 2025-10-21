use std::collections::HashMap;
use super::vehicle::{Vehicle, Direction, Route};


#[derive(Debug)]
pub struct Intersection {
    /// Stores vehicles in each lane, organized by direction
    /// Each direction (North, South, East, West) has its own queue of vehicles
    pub lanes: HashMap<Direction, Vec<Vehicle>>,
    
    /// Minimum safe distance between vehicles (in meters)
    pub safe_distance: f32,
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
    /// 
    /// This is a simplified check based on direction and route.
    /// In a real implementation, this would use more sophisticated path prediction.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intersection_creation() {
        let intersection = Intersection::new(10.0);
        
        assert_eq!(intersection.safe_distance, 10.0);
        assert_eq!(intersection.lanes.len(), 4);
        assert_eq!(intersection.total_vehicles(), 0);
    }

    #[test]
    fn test_add_vehicle() {
        let mut intersection = Intersection::new(10.0);
        
        let vehicle = Vehicle::new(
            1,
            (0.0, -100.0),
            10.0,
            Route::Straight,
            Direction::North,
            100.0,
        );
        
        let success = intersection.add_vehicle(Direction::North, vehicle);
        
        assert!(success);
        assert_eq!(intersection.vehicles_in_lane(Direction::North), 1);
        assert_eq!(intersection.total_vehicles(), 1);
    }

    #[test]
    fn test_multiple_vehicles_same_lane() {
        let mut intersection = Intersection::new(10.0);
        
        // First vehicle far from intersection
        let vehicle1 = Vehicle::new(
            1,
            (0.0, -100.0),
            10.0,
            Route::Straight,
            Direction::North,
            100.0,
        );
        
        // Second vehicle also far but at safe distance
        let vehicle2 = Vehicle::new(
            2,
            (0.0, -120.0),
            10.0,
            Route::Straight,
            Direction::North,
            120.0,
        );
        
        assert!(intersection.add_vehicle(Direction::North, vehicle1));
        assert!(intersection.add_vehicle(Direction::North, vehicle2));
        assert_eq!(intersection.vehicles_in_lane(Direction::North), 2);
    }

    #[test]
    fn test_vehicle_too_close() {
        let mut intersection = Intersection::new(10.0);
        
        // First vehicle
        let vehicle1 = Vehicle::new(
            1,
            (0.0, -100.0),
            10.0,
            Route::Straight,
            Direction::North,
            100.0,
        );
        
        // Second vehicle too close (only 5 meters away)
        let vehicle2 = Vehicle::new(
            2,
            (0.0, -105.0),
            10.0,
            Route::Straight,
            Direction::North,
            105.0,
        );
        
        assert!(intersection.add_vehicle(Direction::North, vehicle1));
        assert!(!intersection.add_vehicle(Direction::North, vehicle2)); // Should fail
        assert_eq!(intersection.vehicles_in_lane(Direction::North), 1);
    }

    #[test]
    fn test_can_enter() {
        let mut intersection = Intersection::new(10.0);
        
        let vehicle1 = Vehicle::new(
            1,
            (0.0, -100.0),
            10.0,
            Route::Straight,
            Direction::North,
            100.0,
        );
        
        let vehicle2 = Vehicle::new(
            2,
            (0.0, -120.0),
            10.0,
            Route::Straight,
            Direction::North,
            120.0,
        );
        
        // First vehicle can always enter empty intersection
        assert!(intersection.can_enter(&vehicle1));
        
        intersection.add_vehicle(Direction::North, vehicle1);
        
        // Second vehicle at safe distance should be able to enter
        assert!(intersection.can_enter(&vehicle2));
    }

    #[test]
    fn test_update_removes_inactive_vehicles() {
        let mut intersection = Intersection::new(10.0);
        
        let vehicle = Vehicle::new(
            1,
            (0.0, -10.0),
            10.0,
            Route::Straight,
            Direction::North,
            10.0,
        );
        
        intersection.add_vehicle(Direction::North, vehicle);
        assert_eq!(intersection.total_vehicles(), 1);
        
        // Update for enough time to pass through intersection
        // Vehicle becomes inactive at distance < -50.0
        intersection.update(10.0); // 10 seconds * 10 m/s = 100m traveled
        
        assert_eq!(intersection.total_vehicles(), 0);
    }

    #[test]
    fn test_update_position() {
        let mut intersection = Intersection::new(10.0);
        
        let vehicle = Vehicle::new(
            1,
            (0.0, -100.0),
            10.0,
            Route::Straight,
            Direction::North,
            100.0,
        );
        
        intersection.add_vehicle(Direction::North, vehicle);
        
        // Update for 2 seconds
        intersection.update(2.0);
        
        if let Some(lane) = intersection.lanes.get(&Direction::North) {
            if let Some(v) = lane.first() {
                assert_eq!(v.position.1, -80.0); // Moved 20 meters north
                assert_eq!(v.distance_to_intersection, 80.0);
            }
        }
    }

    #[test]
    fn test_vehicles_from_different_directions() {
        let mut intersection = Intersection::new(10.0);
        
        let north_vehicle = Vehicle::new(
            1,
            (0.0, -100.0),
            10.0,
            Route::Straight,
            Direction::North,
            100.0,
        );
        
        let east_vehicle = Vehicle::new(
            2,
            (-100.0, 0.0),
            10.0,
            Route::Straight,
            Direction::East,
            100.0,
        );
        
        assert!(intersection.add_vehicle(Direction::North, north_vehicle));
        assert!(intersection.add_vehicle(Direction::East, east_vehicle));
        
        assert_eq!(intersection.vehicles_in_lane(Direction::North), 1);
        assert_eq!(intersection.vehicles_in_lane(Direction::East), 1);
        assert_eq!(intersection.total_vehicles(), 2);
    }

    #[test]
    fn test_total_vehicles_count() {
        let mut intersection = Intersection::new(10.0);
        
        for i in 0..4 {
            let direction = match i {
                0 => Direction::North,
                1 => Direction::South,
                2 => Direction::East,
                _ => Direction::West,
            };
            
            let vehicle = Vehicle::new(
                i,
                (0.0, -100.0 - (i as f32 * 20.0)),
                10.0,
                Route::Straight,
                direction,
                100.0,
            );
            
            intersection.add_vehicle(direction, vehicle);
        }
        
        assert_eq!(intersection.total_vehicles(), 4);
    }
}