//! # Quiet Route - Safe Pathfinding Core Library
//!
//! A safety-aware pedestrian routing engine that finds optimal walking routes
//! by considering both distance and safety factors like streetlight coverage
//! and proximity to police stations.
//!
//! ## Features
//!
//! - **Real-world data integration**: Loads OpenStreetMap data and municipal safety infrastructure
//! - **Smart safety scoring**: Graduated scoring based on proximity to lights and police (0.0-1.0)
//! - **Efficient pathfinding**: A* algorithm with safety-weighted costs
//! - **Scale**: Handles 853k+ intersections and 951k+ street segments
//!
//! ## Architecture
//!
//! The library is organized into four main modules:
//!
//! - [`models`] - Core data structures (Node, Edge, RoadGraph, RoadNetwork)
//! - [`parser`] - OpenStreetMap PBF file loading and graph construction
//! - [`safety`] - Safety scoring system using KD-trees for spatial queries
//! - [`router`] - A* pathfinding with safety-weighted cost function
//!
//! ## Example Usage
//!
//! ```ignore
//! use quiet_core::parser::parse_osm;
//! use quiet_core::router::find_safe_path;
//!
//! // 1. Load the street network
//! let network = parse_osm("data/bengaluru.osm.pbf")?;
//!
//! // 2. Find nodes closest to user coordinates
//! let start = network.find_closest_node(12.976, 77.593)?;
//! let end = network.find_closest_node(12.975, 77.605)?;
//!
//! // 3. Calculate the safest route
//! let (cost, path) = find_safe_path(&network.graph, start, end)?;
//!
//! // 4. Convert to GPS coordinates for visualization
//! let coords = network.path_to_coords(&path);
//! ```
//!
//! ## Performance
//!
//! - **Loading**: ~30-60 seconds for Bangalore's full network
//! - **Routing**: Milliseconds for typical intracity routes (thanks to A* heuristic)
//! - **Safety queries**: O(log n) via KD-tree spatial indexing
//!
//! ## Data Requirements
//!
//! - OpenStreetMap PBF file (e.g., bengaluru.osm.pbf)
//! - KML files with streetlight coordinates (4 files, ~196k points)
//! - KML files with police station coordinates (2 files, ~147 points)

/// Core data models for the road network graph
pub mod models;
/// OpenStreetMap parser for building safety-enriched graphs
pub mod parser;
/// A* pathfinding with safety-weighted costs
pub mod router;
/// Safety scoring using proximity to lights and police
pub mod safety;
