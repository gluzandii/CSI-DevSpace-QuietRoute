use quiet_core::parser::parse_osm;

fn main() {
    // Point this to your actual PBF file path
    let path = "/Users/sushi/Dev/Rust/quiet-route/data/OSM (Open Map Data)/bengaluru.osm.pbf";
    match parse_osm(path) {
        Ok(network) => {
            println!(
                "Success! Graph loaded with {} nodes and {} edges.",
                network.graph.node_count(),
                network.graph.edge_count()
            );
            println!(
                "Lookup maps: {} node coordinates, {} OSM mappings",
                network.node_coords.len(),
                network.osm_to_node.len()
            );
        }
        Err(e) => println!("Error loading graph: {}", e),
    }
}
