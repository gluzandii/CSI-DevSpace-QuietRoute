//! API request and response models.

use serde::{Deserialize, Serialize};

/// Request body for the route finding API
#[derive(Debug, Deserialize)]
pub struct RouteRequest {
    /// Starting latitude
    pub start_lat: f64,
    /// Starting longitude
    pub start_lon: f64,
    /// Ending latitude
    pub end_lat: f64,
    /// Ending longitude
    pub end_lon: f64,
}

/// Successful route response
#[derive(Debug, Serialize)]
pub struct RouteResponse {
    /// GeoJSON LineString feature with route geometry and metadata
    pub geojson: serde_json::Value,
    /// Human-readable message
    pub message: String,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
