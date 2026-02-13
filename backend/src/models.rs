//! API request and response models.

use serde::{Deserialize, Serialize};

/// Request body for the route finding API
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct RouteResponse {
    /// GeoJSON LineString feature with route geometry and metadata
    pub geojson: serde_json::Value,
    /// Human-readable message
    pub message: String,
}

/// Request body for the nearest road endpoint
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NearestRoadRequest {
    /// Latitude
    pub lat: f64,
    /// Longitude
    pub lon: f64,
}

/// Response for the nearest road endpoint
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NearestRoadResponse {
    /// Found nearest road coordinate
    pub coord: Option<NearestRoadCoord>,
    /// Message indicating success or reason for failure
    pub message: String,
}

/// Nearest road coordinate with distance information
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NearestRoadCoord {
    /// Latitude of the nearest road point
    pub lat: f64,
    /// Longitude of the nearest road point
    pub lon: f64,
    /// Distance in meters from the requested point to the nearest road
    pub distance_meters: f64,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
