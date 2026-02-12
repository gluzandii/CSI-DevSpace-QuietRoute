//! KML file parser for extracting geographic coordinates.
//!
//! Parses KML (Keyhole Markup Language) XML files used by Google Earth
//! to extract point coordinates for streetlights and police stations.

use anyhow::Result;
use quick_xml::{Reader, events::Event};
use std::{fs::File, io::BufReader, path::Path};

/// Parses a KML file and extracts all coordinate points.
///
/// Designed for parallel execution - can be called concurrently on multiple files.
/// Searches for `<coordinates>` tags and parses lat/lon pairs.
///
/// # Arguments
/// * `path` - Path to the KML file
/// * `is_police` - Whether this file contains police stations (vs streetlights)
///
/// # Returns
/// A tuple of (coordinate_array, is_police) where coordinates are [lat, lon] pairs
///
/// # KML Format
/// Expects coordinates in "lon,lat,alt" format (note: lon before lat in KML)
/// Example: `<coordinates>77.5,12.9,0.0</coordinates>`
///
/// # Errors
/// Returns an error if the file cannot be opened or parsed
pub fn parse_kml_coordinates(path: &Path, is_police: bool) -> Result<(Vec<[f64; 2]>, bool)> {
    tracing::debug!("Loading safety data from: {}", path.display());

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
