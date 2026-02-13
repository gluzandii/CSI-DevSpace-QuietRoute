//! API route handlers.

use axum::{Json, extract::State};
use quiet_core::router::find_safe_path;

use crate::{
    error::AppError,
    models::{
        NearestRoadCoord, NearestRoadRequest, NearestRoadResponse, RouteMetadata, RouteRequest,
        RouteResponse,
    },
    state::AppState,
};

/// GET / or /health - Health check endpoint
pub async fn health_check() -> &'static str {
    "Quiet Route API is running!"
}

/// Helper function to determine safety rating based on score
fn get_safety_rating(score: f64) -> String {
    match score {
        s if s >= 0.85 => "Very Safe 🟢".to_string(),
        s if s >= 0.7 => "Safe 🟡".to_string(),
        s if s >= 0.55 => "Moderate ⚠️".to_string(),
        s if s >= 0.4 => "Risky 🔴".to_string(),
        _ => "Unsafe ⛔".to_string(),
    }
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

    // Calculate route metadata
    let mut total_distance = 0.0;
    let mut total_safety_score = 0.0;
    let mut lit_segments = 0;
    let mut total_segments = 0;

    // Walk through path nodes and analyze edges
    for window in path.windows(2) {
        let from_idx = window[0];
        let to_idx = window[1];

        if let Some(edge) = state.network.graph.find_edge(from_idx, to_idx) {
            let edge_data = &state.network.graph[edge];
            total_distance += edge_data.distance_meters;
            total_safety_score += edge_data.safety_score;
            if edge_data.is_lit {
                lit_segments += 1;
            }
            total_segments += 1;
        }
    }

    let average_safety_score = if total_segments > 0 {
        total_safety_score / total_segments as f64
    } else {
        0.0
    };

    let lit_percentage = if total_segments > 0 {
        (lit_segments as f64 / total_segments as f64) * 100.0
    } else {
        0.0
    };

    // Get police and light distances from start and end points
    let nearest_police_start = state
        .safety_layer
        .nearest_police_distance(request.start_lat, request.start_lon)
        .unwrap_or(0.0);

    let nearest_police_end = state
        .safety_layer
        .nearest_police_distance(request.end_lat, request.end_lon)
        .unwrap_or(0.0);

    let nearest_light_start = state
        .safety_layer
        .nearest_light_distance(request.start_lat, request.start_lon);

    let nearest_light_end = state
        .safety_layer
        .nearest_light_distance(request.end_lat, request.end_lon);

    // Create metadata
    let metadata = RouteMetadata {
        total_distance_meters: total_distance,
        average_safety_score,
        safety_percentage: average_safety_score * 100.0,
        lit_segments_count: lit_segments,
        total_segments,
        lit_percentage,
        nearest_police_start_meters: nearest_police_start,
        nearest_police_end_meters: nearest_police_end,
        nearest_light_start_meters: nearest_light_start,
        nearest_light_end_meters: nearest_light_end,
        safety_rating: get_safety_rating(average_safety_score),
    };

    // Convert to GeoJSON
    let geojson_str = state
        .network
        .path_to_geojson(&path, cost)
        .map_err(|e| AppError::Internal(format!("Failed to generate GeoJSON: {}", e)))?;

    let geojson: serde_json::Value = serde_json::from_str(&geojson_str)
        .map_err(|e| AppError::Internal(format!("Failed to parse GeoJSON: {}", e)))?;

    // Create the message before moving metadata
    let message = format!(
        "Route found: {:.0}m, Safety Score: {:.1}%",
        total_distance, metadata.safety_percentage
    );

    Ok(Json(RouteResponse {
        geojson,
        metadata,
        message,
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
