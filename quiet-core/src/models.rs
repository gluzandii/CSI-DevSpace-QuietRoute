// quiet_core/src/models.rs
use petgraph::graph::NodeIndex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

// 5. Road Network - Contains the graph and lookup maps for the API
#[derive(Debug, Clone)]
pub struct RoadNetwork {
    pub graph: RoadGraph,
    /// Maps NodeIndex to Coordinate - for converting route results to GeoJSON
    pub node_coords: HashMap<NodeIndex, Coord>,
    /// Maps OSM Node ID to NodeIndex - for debugging and reference
    pub osm_to_node: HashMap<i64, NodeIndex>,
}

impl RoadNetwork {
    /// Finds the closest graph node to the given latitude and longitude
    /// This is essential for the REST API to convert user coordinates to graph nodes
    ///
    /// # Example API Flow:
    /// ```ignore
    /// // 1. Load the network once at startup
    /// let network = parse_osm("data/bengaluru.osm.pbf")?;
    ///
    /// // 2. User requests route from (12.93, 77.61) to (12.95, 77.63)
    /// let start = network.find_closest_node(12.93, 77.61)?;
    /// let end = network.find_closest_node(12.95, 77.63)?;
    ///
    /// // 3. Find the safe route
    /// let (cost, path) = find_safe_path(&network.graph, start, end)?;
    ///
    /// // 4. Convert to GeoJSON coordinates
    /// let coords = network.path_to_coords(&path);
    ///
    /// // 5. Return as GeoJSON LineString
    /// // { "type": "LineString", "coordinates": [[lon, lat], ...] }
    /// ```
    pub fn find_closest_node(&self, lat: f64, lon: f64) -> Option<NodeIndex> {
        self.node_coords
            .iter()
            .min_by(|(_, coord_a), (_, coord_b)| {
                let dist_a = utils::geo::haversine_distance(lat, lon, coord_a.lat, coord_a.lon);
                let dist_b = utils::geo::haversine_distance(lat, lon, coord_b.lat, coord_b.lon);
                dist_a
                    .partial_cmp(&dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(node_idx, _)| *node_idx)
    }

    /// Converts a path (list of NodeIndex) to a list of coordinates for GeoJSON
    pub fn path_to_coords(&self, path: &[NodeIndex]) -> Vec<Coord> {
        path.iter()
            .filter_map(|idx| self.node_coords.get(idx))
            .copied()
            .collect()
    }
}
