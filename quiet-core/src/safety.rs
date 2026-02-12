use anyhow::Result;
use kdtree::KdTree;
use kdtree::distance::squared_euclidean;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use rayon::prelude::*;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

// We use a KD-Tree to store points.
// Type: <DistanceType, ObjectData, PointArray>
type PointTree = KdTree<f64, (), [f64; 2]>;

type KMapFile<P> = (P, bool); // (FilePath, IsPoliceStation)

pub struct SafetyLayer {
    lights: PointTree,
    police: PointTree,
}

impl SafetyLayer {
    pub fn new<P: AsRef<Path> + std::fmt::Display + Send + Sync>(
        paths: Vec<KMapFile<P>>,
    ) -> Result<Self> {
        // Parse all KML files in parallel and collect coordinates
        let results: Result<Vec<_>> = paths
            .into_par_iter()
            .map(|(p, is_police)| Self::parse_kml_coordinates(p.as_ref(), is_police))
            .collect();

        let all_coords = results?;

        // Build the trees sequentially with collected coordinates
        let mut lights = KdTree::new(2);
        let mut police = KdTree::new(2);

        for (coords, is_police) in all_coords {
            for point in coords {
                if is_police {
                    police.add(point, ())?;
                } else {
                    lights.add(point, ())?;
                }
            }
        }

        Ok(Self { lights, police })
    }

    /// Calculates a "Safety Score" (0.0 to 1.0) for a specific coordinate
    pub fn get_safety_score(&self, lat: f64, lon: f64) -> f64 {
        let point = [lat, lon];
        let mut score: f64 = 0.5; // Base score (Neutral)

        // 1. Check Streetlights (Radius: 0.0005 degrees ≈ 50 meters)
        // If we find at least one light nearby, boost the score.
        if let Ok(nearest) = self.lights.nearest(&point, 1, &squared_euclidean) {
            if let Some((dist, _)) = nearest.first() {
                if *dist < 0.00000025 {
                    // roughly 50m squared distance in deg
                    score += 0.3;
                }
            }
        }

        // 2. Check Police Stations (Radius: 0.002 degrees ≈ 200 meters)
        // Being near a police station is a huge safety bonus.
        if let Ok(nearest) = self.police.nearest(&point, 1, &squared_euclidean) {
            if let Some((dist, _)) = nearest.first() {
                if *dist < 0.000004 {
                    // roughly 200m squared distance in deg
                    score += 0.2;
                }
            }
        }

        // Cap the score at 1.0 (Perfectly Safe)
        score.min(1.0_f64)
    }

    /// Parses a KML file and extracts coordinates (can be called in parallel)
    fn parse_kml_coordinates(path: &Path, is_police: bool) -> Result<(Vec<[f64; 2]>, bool)> {
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
}
