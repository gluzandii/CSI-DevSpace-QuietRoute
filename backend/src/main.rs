use anyhow::Result;
use osmpbf::{Element, ElementReader};

fn main() -> Result<()> {
    // 1. Tell Rust where the map file is
    let map_path = "/Users/sushi/Dev/Rust/quiet-route/data/OSM (Open Map Data)/planet_77.356,12.789_77.955,13.168.osm.pbf";

    println!("🗺️  Reading map data from: {}", map_path);
    println!("---------------------------------------");

    // 2. Open the file reader
    let reader = ElementReader::from_path(map_path)?;

    // 3. Initialize counters to track what we find
    let mut nodes = 0;
    let mut ways = 0;

    // 4. Scan through the file
    // The .pbf file is a list of millions of "Elements"
    reader.for_each(|element| {
        match element {
            // "DenseNodes" are just compressed points (Lat, Lon)
            Element::DenseNode(_) => nodes += 1,
            // "Nodes" are standard points
            Element::Node(_) => nodes += 1,
            // "Ways" are streets, paths, or building outlines
            Element::Way(_) => ways += 1,
            _ => {} // Ignore other stuff for now
        }
    })?;

    // 5. Print the results
    println!("✅ Reading Complete!");
    println!("---------------------------------------");
    println!("📍 Total Points (Nodes):   {}", nodes);
    println!("🛣️  Total Streets (Ways):   {}", ways);
    println!("---------------------------------------");

    Ok(())
}
