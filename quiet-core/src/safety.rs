use anyhow::Result;
use kdtree::{KdTree, distance::squared_euclidean};
use rayon::prelude::*;
use std::path::Path;

// We use a KD-Tree to store points.
// Type: <DistanceType, ObjectData, PointArray>
// We store the coordinates as data so we can retrieve them
type PointTree = KdTree<f64, [f64; 2], [f64; 2]>;

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
            .map(|(p, is_police)| utils::kml::parse_kml_coordinates(p.as_ref(), is_police))
            .collect();

        let all_coords = results?;

        // Build the trees sequentially with collected coordinates
        let mut lights = KdTree::new(2);
        let mut police = KdTree::new(2);

        for (coords, is_police) in all_coords {
            for point in coords {
                if is_police {
                    police.add(point, point)?;
                } else {
                    lights.add(point, point)?;
                }
            }
        }

        Ok(Self { lights, police })
    }

    /// Calculates a "Safety Score" (0.0 to 1.0) for a specific coordinate
    pub fn get_safety_score(&self, lat: f64, lon: f64) -> f64 {
        let point = [lat, lon];
        let mut score: f64 = 0.5; // Base score (Neutral)

        // 1. Check Streetlights with graduated scoring based on ACTUAL distance in meters
        if let Ok(nearest) = self.lights.nearest(&point, 1, &squared_euclidean) {
            if let Some((_, nearest_coords)) = nearest.first() {
                let dist_meters =
                    utils::geo::haversine_distance(lat, lon, nearest_coords[0], nearest_coords[1]);

                if dist_meters < 150.0 {
                    // < 150m: Excellent lighting (very close)
                    score += 0.35;
                } else if dist_meters < 300.0 {
                    // 150-300m: Good lighting
                    score += 0.25;
                } else if dist_meters < 500.0 {
                    // 300-500m: Moderate lighting
                    score += 0.15;
                } else if dist_meters < 800.0 {
                    // 500-800m: Weak lighting effect
                    score += 0.05;
                }
            }
        }

        // 2. Check Police Stations with graduated scoring based on ACTUAL distance in meters
        if let Ok(nearest) = self.police.nearest(&point, 1, &squared_euclidean) {
            if let Some((_, nearest_coords)) = nearest.first() {
                let dist_meters =
                    utils::geo::haversine_distance(lat, lon, nearest_coords[0], nearest_coords[1]);

                if dist_meters < 500.0 {
                    // < 500m: Very safe (walking distance)
                    score += 0.25;
                } else if dist_meters < 1000.0 {
                    // 500m-1km: Safe
                    score += 0.15;
                } else if dist_meters < 2000.0 {
                    // 1-2km: Moderately safe
                    score += 0.08;
                } else if dist_meters < 3000.0 {
                    // 2-3km: Slight safety boost
                    score += 0.03;
                }
            }
        }

        // Cap the score at 1.0 (Perfectly Safe)
        score.min(1.0_f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_real_safety() {
        // 1. Load your actual files from the data folder
        let paths = vec![
            // Police KML files
            (
                "../data/KML (Police)/Blr_Urban_Police_station_location.kml",
                true,
            ),
            ("../data/KML (Police)/Blr_Output_Location_Map.kml", true),
            // Light KML files
            ("../data/KML (Lights)/Blr_East_Zone.kml", false),
            ("../data/KML (Lights)/Bommanahali.kml", false),
            ("../data/KML (Lights)/Dasarahali.kml", false),
            ("../data/KML (Lights)/RR_Nagar.kml", false),
        ];

        // Create SafetyLayer with all KML files
        let safety = SafetyLayer::new(paths).unwrap();

        // 2. Test Specific Coordinates

        // Test 1: Cubbon Park Police Station (Should be Safe)
        // Approx Location: 12.976, 77.593
        let score_cubbon = safety.get_safety_score(12.976, 77.593);
        println!(
            "👮 Cubbon Park Area Score: {:.2} (Expected: >0.6)\n",
            score_cubbon
        );

        // Test 2: Nexus Mall Koramangala (Urban commercial area)
        // Approx Location: 12.9352, 77.6245
        let score_nexus = safety.get_safety_score(12.9352, 77.6245);
        println!(
            "🛍️ Nexus Mall Area Score: {:.2} (Expected: mid-range)\n",
            score_nexus
        );

        // Test 2: Random Highway Spot (Likely lower score)
        // Location: 13.100, 77.500
        let score_random = safety.get_safety_score(13.100, 77.500);
        println!(
            "🌑 Random Highway Score:   {:.2} (Expected: Lower than Cubbon)\n",
            score_random
        );

        // Verify Cubbon Park has better safety than random location
        assert!(
            score_cubbon > score_random,
            "Cubbon Park ({}) should have higher score than random highway ({})",
            score_cubbon,
            score_random
        );

        // Ensure scores are within valid range
        assert!(
            score_cubbon >= 0.0 && score_cubbon <= 1.0,
            "Cubbon Park score out of bounds"
        );
        assert!(
            score_random >= 0.0 && score_random <= 1.0,
            "Random location score out of bounds"
        );
    }
}
