//! # Quiet Route Backend - Safe Pathfinding API
//!
//! REST API for the Quiet Route safe pedestrian routing system.
//! Loads the Bangalore street network with safety metadata and provides
//! endpoints to find optimal safe walking routes.
//!
//! ## API Endpoints
//!
//! - `GET /health` - Health check
//! - `POST /route` - Find safe route between two coordinates
//!
//! ## Example Request
//!
//! ```bash
//! curl -X POST http://127.0.0.1:3000/route \
//!   -H "Content-Type: application/json" \
//!   -d '{
//!     "startLat": 12.923782,
//!     "startLon": 77.651635,
//!     "endLat": 12.912297,
//!     "endLon": 77.638196
//!   }'
//! ```
//!
//! Run with: `cargo run --release`

mod error;
mod models;
mod routes;
mod state;

use axum::{http::Method, routing::get, routing::post, Router};
use quiet_core::parser::parse_osm;
use quiet_core::safety::SafetyLayer;
use state::AppState;
use tower_http::cors::{Any, CorsLayer};

// ═══════════════════════════════════════════════════════════════════════════════
// DEMO MAIN - Commented out for API server development
// ═══════════════════════════════════════════════════════════════════════════════
/*
fn main() {
    println!("🚀 Quiet Route - Safe Pathfinding System\n");
    println!("═══════════════════════════════════════════════════════════\n");

    // Load the Bangalore street network
    let path = "/Users/sushi/Dev/Rust/quiet-route/data/OSM (Open Map Data)/bengaluru.osm.pbf";
    println!("📂 Loading map data from: bengaluru.osm.pbf...");

    let network = match parse_osm(path) {
        Ok(net) => {
            println!("✅ Graph loaded successfully!");
            println!("   • {} nodes (intersections)", net.graph.node_count());
            println!("   • {} edges (street segments)", net.graph.edge_count());
            println!("   • {} coordinate lookups", net.node_coords.len());
            println!();
            net
        }
        Err(e) => {
            eprintln!("❌ Error loading graph: {}", e);
            return;
        }
    };

    // Demo: Find a safe route between two locations
    println!("═══════════════════════════════════════════════════════════");
    println!("🧭 Finding Safe Route\n");

    // Example: Broadway HSR 27th Main to McDonald's HSR Layout
    let start_lat = 12.923782;
    let start_lon = 77.651635;
    let end_lat = 12.912297;
    let end_lon = 77.638196;

    println!(
        "📍 Start: Broadway HSR 27th Main ({}, {})",
        start_lat, start_lon
    );
    println!(
        "📍 End:   McDonald's HSR Layout ({}, {})\n",
        end_lat, end_lon
    );

    // Find closest nodes in the graph
    let start_idx = match network.find_closest_node(start_lat, start_lon) {
        Some(idx) => {
            println!("✓ Found start node: {:?}", idx);
            idx
        }
        None => {
            eprintln!("❌ Could not find start node in graph");
            return;
        }
    };

    let end_idx = match network.find_closest_node(end_lat, end_lon) {
        Some(idx) => {
            println!("✓ Found end node: {:?}\n", idx);
            idx
        }
        None => {
            eprintln!("❌ Could not find end node in graph");
            return;
        }
    };

    // Calculate the safe path
    println!("🔍 Calculating safest route using A* algorithm...\n");

    match find_safe_path(&network.graph, start_idx, end_idx) {
        Some((cost, path)) => {
            println!("═══════════════════════════════════════════════════════════");
            println!("✅ ROUTE FOUND!\n");
            println!("📊 Route Statistics:");
            println!("   • Safety-weighted cost: {:.2}", cost);
            println!("   • Number of waypoints: {}", path.len());

            // Convert to coordinates
            let coords = network.path_to_coords(&path);
            println!("   • Total coordinates: {}", coords.len());

            // Calculate actual distance (sum of edge distances)
            let mut total_distance = 0.0;
            for i in 0..path.len() - 1 {
                if let Some(edge) = network.graph.find_edge(path[i], path[i + 1]) {
                    if let Some(edge_weight) = network.graph.edge_weight(edge) {
                        total_distance += edge_weight.distance_meters;
                    }
                }
            }
            println!(
                "   • Actual distance: {:.2} meters ({:.2} km)",
                total_distance,
                total_distance / 1000.0
            );

            // Show first few waypoints
            println!("\n🗺️  Route Preview (First 20 waypoints):");
            for (i, coord) in coords.iter().take(20).enumerate() {
                println!("   {}. Lat: {:.6}, Lon: {:.6}", i + 1, coord.lat, coord.lon);
            }

            if coords.len() > 20 {
                println!("   ...");
                if let Some(last) = coords.last() {
                    println!(
                        "   {}. Lat: {:.6}, Lon: {:.6} (destination)",
                        coords.len(),
                        last.lat,
                        last.lon
                    );
                }
            }

            // Generate GeoJSON output
            println!("\n📍 GeoJSON Output:");
            println!("═══════════════════════════════════════════════════════════");
            match network.path_to_geojson(&path, cost) {
                Ok(geojson) => {
                    println!("{}", geojson);
                }
                Err(e) => {
                    eprintln!("⚠️  Failed to generate GeoJSON: {}", e);
                }
            }
            println!("═══════════════════════════════════════════════════════════");

            println!("\n💡 Ready for API integration!");
            println!("   → Export as GeoJSON LineString");
            println!("   → Send to frontend for map visualization");
            println!("\n═══════════════════════════════════════════════════════════");
        }
        None => {
            println!("❌ No path found between these locations");
            println!("   (This could happen if the points are in disconnected graph components)");
        }
    }
}
*/

#[tokio::main]
async fn main() {
    // Load the Bangalore street network at startup
    let osm_path = "/Users/sushi/Dev/Rust/quiet-route/data/OSM (Open Map Data)/bengaluru.osm.pbf";

    let network = match parse_osm(osm_path) {
        Ok(net) => net,
        Err(_e) => {
            std::process::exit(1);
        }
    };

    // Load the safety layer with KML data
    let safety_paths = vec![
        (
            "/Users/sushi/Dev/Rust/quiet-route/data/KML (Police)/Blr_Urban_Police_station_location.kml",
            true,
        ),
        (
            "/Users/sushi/Dev/Rust/quiet-route/data/KML (Police)/Blr_Output_Location_Map.kml",
            true,
        ),
        (
            "/Users/sushi/Dev/Rust/quiet-route/data/KML (Lights)/Blr_East_Zone.kml",
            false,
        ),
        (
            "/Users/sushi/Dev/Rust/quiet-route/data/KML (Lights)/Bommanahali.kml",
            false,
        ),
        (
            "/Users/sushi/Dev/Rust/quiet-route/data/KML (Lights)/Dasarahali.kml",
            false,
        ),
        (
            "/Users/sushi/Dev/Rust/quiet-route/data/KML (Lights)/RR_Nagar.kml",
            false,
        ),
    ];

    let safety_layer = match SafetyLayer::new(safety_paths) {
        Ok(layer) => layer,
        Err(_e) => {
            std::process::exit(1);
        }
    };

    // Create shared application state
    let state = AppState::new(network, safety_layer);

    // CORS (Cross-Origin Resource Sharing)
    // NOTE: `Any` is fine for local development. For production, replace with a fixed allowlist.
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    // Build the router with routes
    let app = Router::new()
        .route("/", get(routes::health_check))
        .route("/health", get(routes::health_check))
        .route("/route", post(routes::find_route))
        .route("/nearestRoad", post(routes::nearest_road))
        .with_state(state)
        .layer(cors);

    // Start the server
    println!("Binding to http://127.0.0.1:3000");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind to port 3000");

    println!("Server is running at http://127.0.0.1:3000");

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}
