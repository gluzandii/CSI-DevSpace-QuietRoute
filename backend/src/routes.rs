//! API route handlers.

use axum::{Json, extract::State};
use quiet_core::router::find_safe_path;

use crate::{
    error::AppError,
    models::{
        NearestRoadCoord, NearestRoadRequest, NearestRoadResponse, RouteRequest, RouteResponse,
    },
    state::AppState,
};

/// GET / or /health - Health check endpoint
pub async fn health_check() -> &'static str {
    "Quiet Route API is running!"
}

/// POST /route - Find the safest walking route between two coordinates
pub async fn find_route(
    State(state): State<AppState>,
    Json(request): Json<RouteRequest>,
) -> Result<Json<RouteResponse>, AppError> {
    // Find closest nodes to the requested coordinates
    let start_idx = state
        .network
        .find_closest_node(request.start_lat, request.start_lon)
        .ok_or_else(|| {
            AppError::NotFound("Could not find start location in road network".into())
        })?;

    let end_idx = state
        .network
        .find_closest_node(request.end_lat, request.end_lon)
        .ok_or_else(|| AppError::NotFound("Could not find end location in road network".into()))?;

    // Calculate the safe path using A*
    let (cost, path) = find_safe_path(&state.network.graph, start_idx, end_idx)
        .ok_or_else(|| AppError::NotFound("No path found between these locations".into()))?;

    // Convert to GeoJSON
    let geojson_str = state
        .network
        .path_to_geojson(&path, cost)
        .map_err(|e| AppError::Internal(format!("Failed to generate GeoJSON: {}", e)))?;

    let geojson: serde_json::Value = serde_json::from_str(&geojson_str)
        .map_err(|e| AppError::Internal(format!("Failed to parse GeoJSON: {}", e)))?;

    Ok(Json(RouteResponse {
        geojson,
        message: format!("Route found with {} waypoints", path.len()),
    }))
}

/// POST /nearestRoad - Find the nearest road to a given coordinate
pub async fn nearest_road(
    State(state): State<AppState>,
    Json(request): Json<NearestRoadRequest>,
) -> Result<Json<NearestRoadResponse>, AppError> {
    const MAX_DISTANCE_METERS: f64 = 200.0;

    match state
        .network
        .find_nearest_road(request.lat, request.lon, MAX_DISTANCE_METERS)
    {
        Some((coord, distance)) => Ok(Json(NearestRoadResponse {
            coord: Some(NearestRoadCoord {
                lat: coord.lat,
                lon: coord.lon,
                distance_meters: distance,
            }),
            message: format!("Found nearest road {:.2} meters away", distance),
        })),
        None => Ok(Json(NearestRoadResponse {
            coord: None,
            message: format!(
                "No road found within {} meters of the specified coordinates",
                MAX_DISTANCE_METERS
            ),
        })),
    }
}
