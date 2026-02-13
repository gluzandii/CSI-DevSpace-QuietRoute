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

/// Route metadata with safety and infrastructure information
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RouteMetadata {
    /// Total distance of the route in meters
    pub total_distance_meters: f64,
    /// Average safety score of the route (0.0 to 1.0)
    pub average_safety_score: f64,
    /// Safety score as a percentage (0 to 100)
    pub safety_percentage: f64,
    /// Number of lit street segments on the route
    pub lit_segments_count: usize,
    /// Total number of segments in the route
    pub total_segments: usize,
    /// Percentage of lit segments
    pub lit_percentage: f64,
    /// Distance to nearest police station from start (meters)
    pub nearest_police_start_meters: f64,
    /// Distance to nearest police station from end (meters)
    pub nearest_police_end_meters: f64,
    /// Distance to nearest streetlight from start (meters)
    pub nearest_light_start_meters: Option<f64>,
    /// Distance to nearest streetlight from end (meters)
    pub nearest_light_end_meters: Option<f64>,
    /// Human-readable safety rating (e.g., "Very Safe", "Safe", "Moderate", "Unsafe")
    pub safety_rating: String,
}

/// Successful route response with detailed metadata
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteResponse {
    /// GeoJSON LineString feature with route geometry and metadata
    pub geojson: serde_json::Value,
    /// Detailed route metadata and safety information
    pub metadata: RouteMetadata,
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
