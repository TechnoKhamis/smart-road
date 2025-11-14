//! Statistics module for tracking and rendering game stats

pub mod stats;
pub use stats::StatisticsManager;

use std::sync::Mutex;
use once_cell::sync::Lazy;

/// Global singleton instance of StatisticsManager
pub static STATS: Lazy<Mutex<StatisticsManager>> = Lazy::new(|| {
    Mutex::new(StatisticsManager::new())
});
