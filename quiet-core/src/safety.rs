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

    /// Helper to create a SafetyLayer with predefined test data
    fn create_test_safety_layer() -> SafetyLayer {
        let mut lights = KdTree::new(2);
        let mut police = KdTree::new(2);

        // Add a streetlight at coordinates (12.9, 77.5) - Bangalore area
        lights.add([12.9, 77.5], [12.9, 77.5]).unwrap();

        // Add a second light nearby
        lights.add([12.901, 77.501], [12.901, 77.501]).unwrap();

        // Add a police station at (12.95, 77.55)
        police.add([12.95, 77.55], [12.95, 77.55]).unwrap();

        SafetyLayer { lights, police }
    }

    #[test]
    fn test_empty_safety_layer() {
        // Test with no KML files (empty paths)
        let result = SafetyLayer::new(vec![] as Vec<(&str, bool)>);
        assert!(
            result.is_ok(),
            "Should successfully create SafetyLayer with empty paths"
        );

        let safety = result.unwrap();
        let score = safety.get_safety_score(12.9, 77.5);
        assert_eq!(score, 0.5, "Empty layer should return base neutral score");
    }

    #[test]
    fn test_base_score() {
        let safety = create_test_safety_layer();

        // Far away from any lights or police stations
        let score = safety.get_safety_score(20.0, 80.0);
        assert_eq!(
            score, 0.5,
            "Base score should be 0.5 when no features detected"
        );
    }

    #[test]
    fn test_haversine_distance() {
        // Test same point
        let dist = utils::geo::haversine_distance(12.9, 77.5, 12.9, 77.5);
        assert!(dist.abs() < 0.1, "Same point should have nearly 0 distance");

        // Test known distance: ~111km for 1 degree latitude
        let dist_lat = utils::geo::haversine_distance(0.0, 0.0, 1.0, 0.0);
        assert!(
            dist_lat > 110000.0 && dist_lat < 112000.0,
            "1 degree latitude should be ~111km, got: {}",
            dist_lat
        );

        // Test realistic Bangalore distance
        // From Cubbon Park to Electronic City (approximately 20km)
        let dist_bangalore = utils::geo::haversine_distance(12.976, 77.593, 12.845, 77.663);
        assert!(
            dist_bangalore > 15000.0 && dist_bangalore < 25000.0,
            "Distance should be approximately 20km, got: {} meters",
            dist_bangalore
        );
    }

    #[test]
    fn test_score_with_streetlight() {
        let safety = create_test_safety_layer();

        // Very close to the streetlight at (12.9, 77.5)
        // Should be within 150m for excellent lighting bonus
        let score = safety.get_safety_score(12.9001, 77.5001);
        assert!(
            score > 0.5,
            "Score should increase with nearby streetlight. Got: {}",
            score
        );

        // The bonus should be significant for nearby lighting
        assert!(
            score >= 0.75,
            "Score with nearby light should be >= 0.75. Got: {}",
            score
        );
    }

    #[test]
    fn test_score_with_police_station() {
        let safety = create_test_safety_layer();

        // Very close to police station at (12.95, 77.55)
        // Should be within 500m for very safe bonus
        let score = safety.get_safety_score(12.9501, 77.5501);
        assert!(
            score > 0.5,
            "Score should increase with nearby police station. Got: {}",
            score
        );

        // The bonus should be significant for nearby police
        assert!(
            score >= 0.65,
            "Score with police should be >= 0.65. Got: {}",
            score
        );
    }

    #[test]
    fn test_combined_bonuses() {
        let mut lights = KdTree::new(2);
        let mut police = KdTree::new(2);

        // Place both very close together at same location
        lights.add([12.9, 77.5], [12.9, 77.5]).unwrap();
        police.add([12.9, 77.5], [12.9, 77.5]).unwrap();

        let safety = SafetyLayer { lights, police };

        // Should get maximum bonuses (excellent light < 150m + very safe police < 500m)
        let score = safety.get_safety_score(12.9, 77.5);
        assert!(
            score >= 0.95,
            "Score with both light and police at same location should be >= 0.95 (0.5 + 0.35 + 0.25 = 1.0 capped). Got: {}",
            score
        );
    }

    #[test]
    fn test_score_capped_at_one() {
        let mut lights = KdTree::new(2);
        let mut police = KdTree::new(2);

        // Place all at same location
        lights.add([0.0, 0.0], [0.0, 0.0]).unwrap();
        police.add([0.0, 0.0], [0.0, 0.0]).unwrap();

        let safety = SafetyLayer { lights, police };

        let score = safety.get_safety_score(0.0, 0.0);
        assert_eq!(
            score, 1.0,
            "Score should be capped at 1.0 (Perfect safety). Got: {}",
            score
        );
        assert!(
            score <= 1.0,
            "Score should never exceed 1.0. Got: {}",
            score
        );
    }

    #[test]
    fn test_score_never_below_base() {
        let safety = create_test_safety_layer();

        // Any location should score at least 0.5 (base score)
        for lat in 0..20 {
            for lon in 70..80 {
                let score = safety.get_safety_score(lat as f64, lon as f64);
                assert!(
                    score >= 0.5,
                    "Score should never be below base 0.5. Got: {} at ({}, {})",
                    score,
                    lat,
                    lon
                );
            }
        }
    }

    #[test]
    fn test_multiple_lights_detection() {
        let mut lights = KdTree::new(2);
        let police = KdTree::new(2);

        // Add multiple lights at different locations
        lights.add([12.9, 77.5], [12.9, 77.5]).unwrap();
        lights.add([12.91, 77.51], [12.91, 77.51]).unwrap();
        lights.add([12.92, 77.52], [12.92, 77.52]).unwrap();

        let safety = SafetyLayer { lights, police };

        // Should detect the nearest one to each query point
        let score_near_first = safety.get_safety_score(12.9001, 77.5001);
        let score_near_third = safety.get_safety_score(12.9201, 77.5201);

        assert!(score_near_first > 0.5, "Should detect nearby light");
        assert!(score_near_third > 0.5, "Should detect nearby light");
    }

    #[test]
    fn test_out_of_range_no_bonus() {
        let mut lights = KdTree::new(2);
        let mut police = KdTree::new(2);

        lights.add([12.9, 77.5], [12.9, 77.5]).unwrap();
        police.add([12.9, 77.5], [12.9, 77.5]).unwrap();

        let safety = SafetyLayer { lights, police };

        // Far outside the detection radius (over 100km away)
        let score = safety.get_safety_score(13.9, 78.5);
        assert_eq!(
            score, 0.5,
            "Score should be base when features are out of range. Got: {}",
            score
        );
    }

    #[test]
    fn test_score_bounds() {
        let safety = create_test_safety_layer();

        // Test that score is always between 0.0 and 1.0
        for lat in -90..90 {
            for lon in -180..180 {
                let score = safety.get_safety_score(lat as f64, lon as f64);
                assert!(
                    score >= 0.0 && score <= 1.0,
                    "Score {} at ({}, {}) is out of bounds [0.0, 1.0]",
                    score,
                    lat,
                    lon
                );
            }
        }
    }

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
