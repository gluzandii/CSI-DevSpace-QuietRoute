use anyhow::Result;
use osmpbf::{Element, ElementReader};
use petgraph::graph::NodeIndex;
use std::{collections::HashMap, path::Path};

// Import the models we created in Step 3
use crate::models::{Coord, Edge, Node, RoadGraph};

// --- Helpers ---

fn is_walkable(tag: &str) -> bool {
    matches!(
        tag,
        "residential"
            | "service"
            | "living_street"
            | "pedestrian"
            | "footway"
            | "steps"
            | "path"
            | "cycleway"
            | "primary"
            | "secondary"
            | "tertiary"
            | "unclassified"
    )
}

fn haversine_dist(c1: &Coord, c2: &Coord) -> f64 {
    let r = 6371000.0;
    let dlat = (c2.lat - c1.lat).to_radians();
    let dlon = (c2.lon - c1.lon).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + c1.lat.to_radians().cos() * c2.lat.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    r * c
}

pub fn parse_osm<P: AsRef<Path>>(file_path: P) -> Result<RoadGraph> {
    let file_path = file_path.as_ref();
    tracing::debug!("Loading map from: {}", file_path.display());

    // 1. Initialize the Graph and lookups
    let mut graph = RoadGraph::new_undirected();

    // Maps OSM ID (i64) -> Lat/Lon (Coord)
    let mut coords: HashMap<i64, Coord> = HashMap::with_capacity(500_000);
    // Maps OSM ID (i64) -> PetGraph NodeIndex (u32)
    let mut node_indices: HashMap<i64, NodeIndex> = HashMap::with_capacity(100_000);

    let reader = ElementReader::from_path(file_path)?;

    // 2. Iterate through the PBF file
    // PBF files are ordered: Nodes come first, then Ways.
    reader.for_each(|element| {
        match element {
            // CASE A: It's a Point (Node) -> Store coordinate
            Element::DenseNode(node) => {
                coords.insert(
                    node.id,
                    Coord {
                        lat: node.lat(),
                        lon: node.lon(),
                    },
                );
            }
            Element::Node(node) => {
                coords.insert(
                    node.id(),
                    Coord {
                        lat: node.lat(),
                        lon: node.lon(),
                    },
                );
            }

            // CASE B: It's a Street (Way) -> Add to Graph
            Element::Way(way) => {
                // 1. Check if it's a walkable highway
                let mut highway_type = None;
                for (key, val) in way.tags() {
                    if key == "highway" {
                        highway_type = Some(val);
                        break;
                    }
                }

                if let Some(h_type) = highway_type {
                    if is_walkable(h_type) {
                        // 2. Get the list of Node IDs in this street
                        let refs: Vec<i64> = way.refs().collect();

                        // 3. Connect them pairwise (Node A -> Node B)
                        for window in refs.windows(2) {
                            let start_id = window[0];
                            let end_id = window[1];

                            // Ensure we have coordinates for both
                            if let (Some(start_coord), Some(end_coord)) =
                                (coords.get(&start_id), coords.get(&end_id))
                            {
                                // Get or Create Graph Nodes
                                let u_idx = *node_indices.entry(start_id).or_insert_with(|| {
                                    graph.add_node(Node {
                                        id: start_id as u64,
                                        coord: *start_coord,
                                    })
                                });
                                let v_idx = *node_indices.entry(end_id).or_insert_with(|| {
                                    graph.add_node(Node {
                                        id: end_id as u64,
                                        coord: *end_coord,
                                    })
                                });

                                // Calculate Distance (Cost)
                                let dist = haversine_dist(start_coord, end_coord);

                                // Create the Edge with Default Safety (We will update this later)
                                let edge_data = Edge {
                                    distance_meters: dist,
                                    safety_score: 1.0, // Default: Assume safe
                                    is_lit: false,     // Default: Assume dark
                                    street_type: h_type.to_string(),
                                };

                                graph.add_edge(u_idx, v_idx, edge_data);
                            }
                        }
                    }
                }
            }
            _ => {} // Ignore relations
        }
    })?;

    tracing::info!(
        "Graph has been built. Notes: {}, Edges: {}",
        graph.node_count(),
        graph.edge_count()
    );
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_osm_with_nonexistent_file() {
        let result = parse_osm("nonexistent_file.osm.pbf");
        assert!(result.is_err(), "Should fail with nonexistent file");
    }

    #[test]
    fn test_parse_osm_with_empty_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"").unwrap();
        temp_file.flush().unwrap();

        let result = parse_osm(temp_file.path());
        // Empty file should either error or return empty graph
        match result {
            Ok(graph) => {
                assert_eq!(
                    graph.node_count(),
                    0,
                    "Empty file should produce empty graph"
                );
                assert_eq!(
                    graph.edge_count(),
                    0,
                    "Empty file should produce empty graph"
                );
            }
            Err(_) => {
                // Also acceptable - parsing empty file may error
            }
        }
    }

    #[test]
    fn test_parse_osm_with_real_file() {
        // This test requires an actual OSM PBF file
        // If the file exists in the workspace, test it
        let test_file = "../data/OSM (Open Map Data)/bengaluru.osm.pbf";

        if std::path::Path::new(test_file).exists() {
            let result = parse_osm(test_file);
            assert!(result.is_ok(), "Should successfully parse valid OSM file");

            let graph = result.unwrap();
            assert!(graph.node_count() > 0, "Graph should contain nodes");
            assert!(graph.edge_count() > 0, "Graph should contain edges");

            // Verify graph is undirected
            assert!(!graph.is_directed(), "Graph should be undirected");
        }
    }

    #[test]
    fn test_haversine_dist_calculation() {
        // Test distance between two known points
        let coord1 = Coord { lat: 0.0, lon: 0.0 };
        let coord2 = Coord { lat: 0.0, lon: 0.0 };

        let dist = haversine_dist(&coord1, &coord2);
        assert_eq!(dist, 0.0, "Distance between same point should be 0");

        // Test with actual coordinates (approximately 1 degree apart)
        let coord3 = Coord { lat: 0.0, lon: 0.0 };
        let coord4 = Coord { lat: 1.0, lon: 0.0 };

        let dist2 = haversine_dist(&coord3, &coord4);
        assert!(
            dist2 > 110000.0 && dist2 < 112000.0,
            "Distance should be approximately 111km for 1 degree latitude"
        );
    }

    #[test]
    fn test_is_walkable() {
        // Test walkable road types
        assert!(is_walkable("residential"), "residential should be walkable");
        assert!(is_walkable("pedestrian"), "pedestrian should be walkable");
        assert!(is_walkable("footway"), "footway should be walkable");
        assert!(is_walkable("primary"), "primary should be walkable");
        assert!(is_walkable("cycleway"), "cycleway should be walkable");

        // Test non-walkable road types
        assert!(!is_walkable("motorway"), "motorway should not be walkable");
        assert!(!is_walkable("trunk"), "trunk should not be walkable");
        assert!(!is_walkable("rail"), "rail should not be walkable");
    }

    #[test]
    fn test_parse_osm_graph_properties() {
        let test_file = "../data/OSM (Open Map Data)/bengaluru.osm.pbf";

        if std::path::Path::new(test_file).exists() {
            let result = parse_osm(test_file);

            if let Ok(graph) = result {
                // Verify all edges have positive distances
                for edge in graph.edge_indices() {
                    if let Some(edge_weight) = graph.edge_weight(edge) {
                        assert!(
                            edge_weight.distance_meters >= 0.0,
                            "Edge distance should be non-negative"
                        );
                        assert!(
                            edge_weight.safety_score == 1.0,
                            "Default safety score should be 1.0"
                        );
                        assert!(!edge_weight.is_lit, "Default is_lit should be false");
                    }
                }

                // Verify all nodes have valid coordinates
                for node in graph.node_indices() {
                    if let Some(node_weight) = graph.node_weight(node) {
                        assert!(
                            node_weight.coord.lat >= -90.0 && node_weight.coord.lat <= 90.0,
                            "Latitude should be in valid range"
                        );
                        assert!(
                            node_weight.coord.lon >= -180.0 && node_weight.coord.lon <= 180.0,
                            "Longitude should be in valid range"
                        );
                    }
                }
            }
        }
    }
}
