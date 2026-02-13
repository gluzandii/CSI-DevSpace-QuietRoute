//! Application state shared across all request handlers.

use quiet_core::models::RoadNetwork;
use std::sync::Arc;

/// Shared application state containing the loaded road network
#[derive(Clone)]
pub struct AppState {
    pub network: Arc<RoadNetwork>,
}

impl AppState {
    pub fn new(network: RoadNetwork) -> Self {
        Self {
            network: Arc::new(network),
        }
    }
}
