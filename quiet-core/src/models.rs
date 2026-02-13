//! Data models for the road network graph.
//!
//! This module defines the core data structures used to represent a walkable
//! street network with safety metadata. The graph is built from OpenStreetMap data
//! and enriched with safety scores calculated from streetlight and police station locations.

use petgraph::graph::NodeIndex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A geographic coordinate representing a point on Earth.
///
/// Uses the WGS84 coordinate system (standard GPS coordinates).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Coord {
    /// Latitude in decimal degrees (-90 to +90)
    pub lat: f64,
    /// Longitude in decimal degrees (-180 to +180)
    pub lon: f64,
}

/// A node in the street network graph, representing an intersection or waypoint.
///
/// Each node corresponds to a point in the OpenStreetMap data where streets meet
/// or change direction. Nodes are connected by edges (street segments).
#[derive(Debug, Clone)]
pub struct Node {
    /// Unique identifier from OpenStreetMap (uses u64 for large OSM IDs)
    pub id: u64,
    /// Geographic location of this intersection
    pub coord: Coord,
}

/// An edge in the street network graph, representing a walkable street segment.
///
/// Edges connect two nodes and contain metadata about the street's physical properties
/// and calculated safety characteristics. Each edge has been analyzed using proximity
/// to streetlights and police stations to generate a safety score.
#[derive(Debug, Clone)]
pub struct Edge {
    /// Physical length of this street segment in meters
    pub distance_meters: f64,
    /// Safety score from 0.0 (dangerous) to 1.0 (safe)
    ///
    /// Calculated based on proximity to streetlights and police stations.
    /// Higher scores indicate better lighting and closer emergency services.
    pub safety_score: f64,
    /// Whether this street segment has streetlight coverage within 500 meters
    pub is_lit: bool,
    /// OpenStreetMap highway type (e.g., "residential", "primary", "footway")
    pub street_type: String,
}

/// Type alias for the undirected graph structure.
///
/// Uses petgraph's UnGraph with Node data at vertices and Edge data on connections.
/// Undirected because streets in Bangalore are bidirectional.
pub type RoadGraph = petgraph::graph::UnGraph<Node, Edge>;

/// Complete road network with graph and lookup tables for fast coordinate-based queries.
///
/// This is the primary data structure returned by the OSM parser and used for routing.
/// It contains the graph itself plus optimized lookup maps for API integration.
#[derive(Debug, Clone)]
pub struct RoadNetwork {
    /// The street network graph with safety-enriched edges
    pub graph: RoadGraph,
    /// Maps internal NodeIndex to geographic coordinates
    ///
    /// Used to convert routing results back to GPS coordinates for GeoJSON output.
    pub node_coords: HashMap<NodeIndex, Coord>,
    /// Maps original OSM node IDs to internal graph indices
    ///
    /// Useful for debugging and cross-referencing with OpenStreetMap data.
    pub osm_to_node: HashMap<i64, NodeIndex>,
}

impl RoadNetwork {
    /// Finds the closest graph node to a given GPS coordinate.
    ///
    /// This is essential for API integration - users provide lat/lon coordinates,
    /// and we need to map those to actual nodes in our street network graph.
    ///
    /// # Arguments
    /// * `lat` - Latitude in decimal degrees
    /// * `lon` - Longitude in decimal degrees
    ///
    /// # Returns
    /// The NodeIndex of the closest intersection, or None if the graph is empty.
    ///
    /// # Performance
    /// O(n) linear search through all nodes. For production, consider using a spatial index.
    ///
    /// # Example API Flow:
    /// ```ignore
    /// // 1. Load the network once at startup
    /// let network = parse_osm("/Users/sushi/Dev/Rust/quiet-route/data/OSM (Open Map Data)/bengaluru.osm.pbf")?;
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

    /// Converts a routing path to a list of GPS coordinates.
    ///
    /// Takes the output of pathfinding (a sequence of node indices) and converts
    /// it to geographic coordinates suitable for GeoJSON LineString output.
    ///
    /// # Arguments
    /// * `path` - Sequence of node indices from start to destination
    ///
    /// # Returns
    /// Vector of coordinates representing the route waypoints.
    pub fn path_to_coords(&self, path: &[NodeIndex]) -> Vec<Coord> {
        path.iter()
            .filter_map(|idx| self.node_coords.get(idx))
            .copied()
            .collect()
    }

    /// Finds the nearest road node within a maximum distance threshold.
    ///
    /// This is useful for API clients that want to snap a user-provided coordinate
    /// to the nearest actual road in the network. If no road is found within the
    /// threshold distance, returns None.
    ///
    /// # Arguments
    /// * `lat` - Latitude in decimal degrees
    /// * `lon` - Longitude in decimal degrees
    /// * `max_distance_meters` - Maximum acceptable distance to the nearest road
    ///
    /// # Returns
    /// A tuple of (coordinate, distance_in_meters) if a road is found within the threshold,
    /// or None if no road is close enough.
    ///
    /// # Example
    /// ```ignore
    /// let result = network.find_nearest_road(12.93, 77.61, 500.0)?;
    /// // Result: (Coord { lat: 12.931, lon: 77.611 }, 123.45)
    /// ```
    pub fn find_nearest_road(&self, lat: f64, lon: f64, max_distance_meters: f64) -> Option<(Coord, f64)> {
        self.node_coords
            .iter()
            .map(|(_, coord)| {
                let distance = utils::geo::haversine_distance(lat, lon, coord.lat, coord.lon);
                (*coord, distance)
            })
            .filter(|(_, distance)| distance <= &max_distance_meters)
            .min_by(|(_, dist_a), (_, dist_b)| {
                dist_a
                    .partial_cmp(dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Converts a routing path to GeoJSON LineString format.
    ///
    /// Creates a GeoJSON LineString feature representing the route, ready for
    /// map visualization or API responses. GeoJSON coordinates use [lon, lat] order.
    ///
    /// # Arguments
    /// * `path` - Sequence of node indices from start to destination
    /// * `safety_cost` - The safety-weighted cost from the routing algorithm
    ///
    /// # Returns
    /// GeoJSON string representing the route as a LineString
    ///
    /// # Example Output
    /// ```json
    /// {
    ///   "type": "Feature",
    ///   "geometry": {
    ///     "type": "LineString",
    ///     "coordinates": [[12.976, 77.593], [12.975, 77.594], ...]
    ///   },
    ///   "properties": {
    ///     "waypoints": 84,
    ///     "safety_cost": 1508.42,
    ///     "distance_meters": 1453.23,
    ///     "distance_km": 1.45
    ///   }
    /// }
    /// ```
    pub fn path_to_geojson(
        &self,
        path: &[NodeIndex],
        safety_cost: f64,
    ) -> Result<String, Box<dyn std::error::Error>> {
        use geojson::{Feature, Geometry, Value};
        use serde_json::json;

        // Convert path to coordinates
        let coords = self.path_to_coords(path);

        // Calculate actual distance by summing edge lengths
        let mut total_distance = 0.0;
        for i in 0..path.len() - 1 {
            if let Some(edge) = self.graph.find_edge(path[i], path[i + 1]) {
                if let Some(edge_weight) = self.graph.edge_weight(edge) {
                    total_distance += edge_weight.distance_meters;
                }
            }
        }

        // Using [lat, lon] order
        let positions: Vec<Vec<f64>> = coords
            .iter()
            .map(|coord| vec![coord.lat, coord.lon])
            .collect();

        // Create LineString geometry
        let geometry = Geometry::new(Value::LineString(positions));

        // Create Feature with properties
        let mut feature = Feature {
            bbox: None,
            geometry: Some(geometry),
            id: None,
            properties: None,
            foreign_members: None,
        };

        // Add properties
        feature.properties = Some(
            json!({
                "waypoints": coords.len(),
                "routeType": "safe_pedestrian",
                "safetyCost": format!("{:.2}", safety_cost),
                "distanceMeters": format!("{:.2}", total_distance),
                "distanceKm": format!("{:.2}", total_distance / 1000.0),
                "costDistanceRatio": format!("{:.2}", safety_cost / total_distance.max(1.0))
            })
            .as_object()
            .unwrap()
            .clone(),
        );

        // Serialize to JSON string
        Ok(serde_json::to_string_pretty(&feature)?)
    }
}
