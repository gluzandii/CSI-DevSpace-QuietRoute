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

pub fn parse_osm<P: AsRef<Path> + std::fmt::Display>(file_path: P) -> Result<RoadGraph> {
    tracing::debug!("Loading map from: {file_path}");

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
