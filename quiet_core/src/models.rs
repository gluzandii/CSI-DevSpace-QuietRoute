// quiet_core/src/models.rs
use serde::{Deserialize, Serialize};

// 1. A Coordinate (Just a point on Earth)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Coord {
    pub lat: f64,
    pub lon: f64,
}

// 2. A Node (An intersection in the graph)
// We use u64 for OSM IDs (they are huge numbers)
#[derive(Debug, Clone)]
pub struct Node {
    pub id: u64,
    pub coord: Coord,
}

// 3. An Edge (A street segment connecting two nodes)
#[derive(Debug, Clone)]
pub struct Edge {
    pub distance_meters: f64,
    pub safety_score: f64,   // 0.0 (Dangerous) to 1.0 (Safe)
    pub is_lit: bool,        // From your Light Data
    pub street_type: String, // "residential", "primary", etc.
}

// 4. The Graph Type
// We alias the complex PetGraph type to something simple
pub type RoadGraph = petgraph::graph::UnGraph<Node, Edge>;
