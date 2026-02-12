use anyhow::Result;
use osmpbf::{Element, ElementReader};
use petgraph::graph::{NodeIndex, UnGraph};
use std::collections::HashMap;

// A simple struct to hold our Node Data (Lat/Lon)
struct NodeData {
    lat: f64,
    lon: f64,
}

fn get_all_data() -> Result<()> {
    // ⚠️ CHECK THIS PATH: Ensure it points to your actual file!
    let map_path = "/Users/sushi/Dev/Rust/quiet-route/data/OSM (Open Map Data)/planet_77.356,12.789_77.955,13.168.osm.pbf";

    println!("🏗️  Building the QuietRoute Graph...");

    // 1. DATA STRUCTURES
    let mut coords: HashMap<i64, NodeData> = HashMap::with_capacity(1_000_000);
    let mut node_indices: HashMap<i64, NodeIndex> = HashMap::new();
    let mut graph = UnGraph::<(), f64>::new_undirected();

    let reader = ElementReader::from_path(map_path)?;

    // 2. READ THE MAP
    let mut way_count = 0;

    reader.for_each(|element| {
        match element {
            // CASE A: Compressed Nodes (DenseNodes)
            Element::DenseNode(node) => {
                coords.insert(
                    node.id,
                    NodeData {
                        lat: node.lat(),
                        lon: node.lon(),
                    },
                );
            }
            // CASE B: Standard Nodes
            Element::Node(node) => {
                coords.insert(
                    node.id(),
                    NodeData {
                        lat: node.lat(),
                        lon: node.lon(),
                    },
                );
            }
            // CASE C: Ways (Streets)
            Element::Way(way) => {
                // --- THE FIX IS HERE ---
                // We must loop through tags to find "highway"
                let mut highway_type = None;

                for (key, value) in way.tags() {
                    if key == "highway" {
                        highway_type = Some(value);
                        break;
                    }
                }

                // If we found a highway tag, check if it is walkable
                if let Some(highway) = highway_type {
                    if is_walkable(highway) {
                        way_count += 1;

                        let refs: Vec<i64> = way.refs().collect();

                        // Connect nodes in the graph
                        for window in refs.windows(2) {
                            let start_id = window[0];
                            let end_id = window[1];

                            if coords.contains_key(&start_id) && coords.contains_key(&end_id) {
                                let u = *node_indices
                                    .entry(start_id)
                                    .or_insert_with(|| graph.add_node(()));
                                let v = *node_indices
                                    .entry(end_id)
                                    .or_insert_with(|| graph.add_node(()));

                                let start_node = coords.get(&start_id).unwrap();
                                let end_node = coords.get(&end_id).unwrap();
                                let dist = distance(
                                    start_node.lat,
                                    start_node.lon,
                                    end_node.lat,
                                    end_node.lon,
                                );

                                graph.add_edge(u, v, dist);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    })?;

    println!("---------------------------------------");
    println!("✅ Graph Built Successfully!");
    println!("📍 Nodes (Intersections): {}", graph.node_count());
    println!("🛣️  Edges (Street Segments): {}", graph.edge_count());
    println!("---------------------------------------");

    Ok(())
}

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

fn distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6371000.0;
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    r * c
}

fn main() {
    get_all_data().unwrap();
}
