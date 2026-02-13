//! Application state shared across all request handlers.

use quiet_core::models::RoadNetwork;
use quiet_core::safety::SafetyLayer;
use std::sync::Arc;

/// Shared application state containing the loaded road network
#[derive(Clone)]
pub struct AppState {
    pub network: Arc<RoadNetwork>,
    pub safety_layer: Arc<SafetyLayer>,
}

impl AppState {
    pub fn new(network: RoadNetwork, safety_layer: SafetyLayer) -> Self {
        Self {
            network: Arc::new(network),
            safety_layer: Arc::new(safety_layer),
        }
    }
}
