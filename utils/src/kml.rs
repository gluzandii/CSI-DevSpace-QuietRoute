use anyhow::Result;
use quick_xml::{Reader, events::Event};
use std::{fs::File, io::BufReader, path::Path};

/// Parses a KML file and extracts coordinates (can be called in parallel)
pub fn parse_kml_coordinates(path: &Path, is_police: bool) -> Result<(Vec<[f64; 2]>, bool)> {
    println!("🔦 Loading safety data from: {}", path.display());

    let file = File::open(path)?;
    let file = BufReader::new(file);
    let mut reader = Reader::from_reader(file);
    let mut buf = Vec::new();
    let mut coords = Vec::new();
    let mut in_coord = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                if e.name().as_ref() == b"coordinates" {
                    in_coord = true;
                }
            }
            Ok(Event::Text(e)) => {
                if in_coord {
                    let text = String::from_utf8_lossy(e.as_ref());
                    // KML coordinates format: "lon,lat,alt" (Example: "77.5,12.9,0.0")
                    let parts: Vec<&str> = text.trim().split(',').collect();

                    if parts.len() >= 2 {
                        // Note: KML is usually Lon, Lat. We want Lat, Lon for our tree.
                        let lon = parts[0].parse::<f64>().unwrap_or(0.0);
                        let lat = parts[1].parse::<f64>().unwrap_or(0.0);

                        if lat != 0.0 && lon != 0.0 {
                            coords.push([lat, lon]);
                        }
                    }
                }
            }
            Ok(Event::End(e)) => {
                if e.name().as_ref() == b"coordinates" {
                    in_coord = false;
                }
            }
            Ok(Event::Eof) => break,
            _ => (),
        }
        buf.clear();
    }
    Ok((coords, is_police))
}
