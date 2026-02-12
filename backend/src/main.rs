use quiet_core::parser::parse_osm;

fn main() {
    // Point this to your actual PBF file path
    let path = "data/OSM (Open Map Data)/planet_77.356,12.789_77.955,13.168.osm.pbf";
    match parse_osm(path) {
        Ok(graph) => println!("Success! Graph loaded with {} edges.", graph.edge_count()),
        Err(e) => println!("Error loading graph: {}", e),
    }
}
