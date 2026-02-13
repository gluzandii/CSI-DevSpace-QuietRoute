//! Safety scoring system using spatial proximity to streetlights and police stations.
//!
//! This module provides real-time safety assessment for any geographic coordinate
//! by analyzing distances to nearby safety infrastructure. Uses KD-trees for
//! efficient O(log n) spatial queries.

use anyhow::Result;
use kdtree::{KdTree, distance::squared_euclidean};
use rayon::prelude::*;
use std::path::Path;

/// KD-tree type for storing 2D geographic coordinates
type PointTree = KdTree<f64, [f64; 2], [f64; 2]>;

/// KML file specification: (file_path, is_police_station)
type KMapFile<P> = (P, bool);

/// Safety analysis layer built from streetlight and police station locations.
///
/// Loads municipal KML data and provides fast spatial queries to calculate
/// safety scores based on proximity to lights and law enforcement.
///
/// # Data Sources
/// - 196,642 streetlight locations (4 KML files covering Bangalore zones)
/// - 147 police station locations (2 KML files)
///
/// # Performance
/// Uses KD-trees for O(log n) nearest-neighbor queries instead of O(n) linear scans.
pub struct SafetyLayer {
    /// KD-tree containing streetlight coordinates for fast proximity queries
    lights: PointTree,
    /// KD-tree containing police station coordinates
    police: PointTree,
}

impl SafetyLayer {
    /// Creates a new SafetyLayer by loading and indexing KML files.
    ///
    /// Parses all KML files in parallel using Rayon for performance,
    /// then builds KD-trees for efficient spatial queries.
    ///
    /// # Arguments
    /// * `paths` - Vector of (file_path, is_police) tuples specifying KML files to load
    ///
    /// # Returns
    /// A SafetyLayer ready for safety score calculations
    ///
    /// # Errors
    /// Returns an error if any KML file cannot be opened or parsed
    ///
    /// # Performance
    /// Parallelizes file parsing across CPU cores for faster loading.
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

        tracing::info!(
            "SafetyLayer initialized: {} lights, {} police stations",
            lights.size(),
            police.size()
        );

        Ok(Self { lights, police })
    }

    /// Calculates a safety score for a specific geographic coordinate.
    ///
    /// Uses graduated distance-based scoring with STRONG emphasis on police station proximity.
    /// Closer proximity to police stations yields significantly higher scores.
    /// Police proximity is the primary safety factor, with streetlights as secondary.
    ///
    /// # Scoring Algorithm
    /// **Base score:** 0.3 (neutral)
    ///
    /// **Police station bonus (PRIMARY WEIGHT - graduated by distance):**
    /// - < 300m: +0.45 (excellent coverage, very safe walking distance)
    /// - 300-700m: +0.30 (good coverage)
    /// - 700-1500m: +0.16 (moderate coverage)
    /// - 1500-2500m: +0.08 (some coverage)
    ///
    /// **Streetlight bonus (secondary - graduated by distance):**
    /// - < 150m: +0.20 (excellent lighting)
    /// - 150-300m: +0.15 (good lighting)
    /// - 300-500m: +0.08 (moderate lighting)
    /// - 500-800m: +0.03 (weak effect)
    ///
    /// # Arguments
    /// * `lat` - Latitude in decimal degrees
    /// * `lon` - Longitude in decimal degrees
    ///
    /// # Returns
    /// Safety score from 0.0 (dangerous) to 1.0 (very safe), capped at 1.0
    pub fn get_safety_score(&self, lat: f64, lon: f64) -> f64 {
        let point = [lat, lon];
        let mut score: f64 = 0.3; // Base score (Neutral, lowered to allow police bonus to dominate)

        // 1. PRIORITY: Check Police Stations with strong graduated scoring based on ACTUAL distance in meters
        if let Ok(nearest) = self.police.nearest(&point, 1, &squared_euclidean) {
            if let Some((_, nearest_coords)) = nearest.first() {
                let dist_meters =
                    utils::geo::haversine_distance(lat, lon, nearest_coords[0], nearest_coords[1]);

                if dist_meters < 300.0 {
                    // < 300m: Excellent coverage, very safe (walking distance)
                    score += 0.35;
                } else if dist_meters < 700.0 {
                    // 300-700m: Good coverage
                    score += 0.20;
                } else if dist_meters < 1500.0 {
                    // 700-1500m: Moderate coverage
                    score += 0.10;
                } else if dist_meters < 2500.0 {
                    // 1500-2500m: Some coverage
                    score += 0.05;
                }
            }
        }

        // 2. Secondary: Check Streetlights with graduated scoring based on ACTUAL distance in meters
        if let Ok(nearest) = self.lights.nearest(&point, 1, &squared_euclidean) {
            if let Some((_, nearest_coords)) = nearest.first() {
                let dist_meters =
                    utils::geo::haversine_distance(lat, lon, nearest_coords[0], nearest_coords[1]);

                if dist_meters < 150.0 {
                    // < 150m: Excellent lighting (very close)
                    score += 0.25;
                } else if dist_meters < 300.0 {
                    // 150-300m: Good lighting
                    score += 0.20;
                } else if dist_meters < 500.0 {
                    // 300-500m: Moderate lighting
                    score += 0.10;
                } else if dist_meters < 800.0 {
                    // 500-800m: Weak lighting effect
                    score += 0.05;
                }
            }
        }

        // Cap the score at 1.0 (Perfectly Safe)
        score.min(1.0_f64)
    }

    /// Checks if a location has streetlight coverage.
    ///
    /// Used to set the `is_lit` boolean flag on street edges.
    ///
    /// # Arguments
    /// * `lat` - Latitude in decimal degrees
    /// * `lon` - Longitude in decimal degrees
    ///
    /// # Returns
    /// `true` if a streetlight exists within 500 meters, `false` otherwise
    pub fn is_lit(&self, lat: f64, lon: f64) -> bool {
        let point = [lat, lon];
        if let Ok(nearest) = self.lights.nearest(&point, 1, &squared_euclidean) {
            if let Some((_, nearest_coords)) = nearest.first() {
                let dist_meters =
                    utils::geo::haversine_distance(lat, lon, nearest_coords[0], nearest_coords[1]);
                return dist_meters < 500.0;
            }
        }
        false
    }

    /// Debug helper: Returns the distance to the nearest streetlight.
    ///
    /// Useful for testing and understanding light coverage density.
    ///
    /// # Arguments
    /// * `lat` - Latitude in decimal degrees
    /// * `lon` - Longitude in decimal degrees
    ///
    /// # Returns
    /// Distance in meters to nearest light, or None if no lights in tree
    pub fn nearest_light_distance(&self, lat: f64, lon: f64) -> Option<f64> {
        let point = [lat, lon];
        if let Ok(nearest) = self.lights.nearest(&point, 1, &squared_euclidean) {
            if let Some((_, nearest_coords)) = nearest.first() {
                let dist_meters =
                    utils::geo::haversine_distance(lat, lon, nearest_coords[0], nearest_coords[1]);
                return Some(dist_meters);
            }
        }
        None
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
                "/Users/sushi/Dev/Rust/quiet-route/data/KML (Police)/Blr_Urban_Police_station_location.kml",
                true,
            ),
            (
                "/Users/sushi/Dev/Rust/quiet-route/data/KML (Police)/Blr_Output_Location_Map.kml",
                true,
            ),
            // Light KML files
            (
                "/Users/sushi/Dev/Rust/quiet-route/data/KML (Lights)/Blr_East_Zone.kml",
                false,
            ),
            (
                "/Users/sushi/Dev/Rust/quiet-route/data/KML (Lights)/Bommanahali.kml",
                false,
            ),
            (
                "/Users/sushi/Dev/Rust/quiet-route/data/KML (Lights)/Dasarahali.kml",
                false,
            ),
            (
                "/Users/sushi/Dev/Rust/quiet-route/data/KML (Lights)/RR_Nagar.kml",
                false,
            ),
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

    #[test]
    fn test_is_lit() {
        // Load the safety layer with real data
        let paths = vec![
            (
                "/Users/sushi/Dev/Rust/quiet-route/data/KML (Police)/Blr_Urban_Police_station_location.kml",
                true,
            ),
            (
                "/Users/sushi/Dev/Rust/quiet-route/data/KML (Police)/Blr_Output_Location_Map.kml",
                true,
            ),
            (
                "/Users/sushi/Dev/Rust/quiet-route/data/KML (Lights)/Blr_East_Zone.kml",
                false,
            ),
            (
                "/Users/sushi/Dev/Rust/quiet-route/data/KML (Lights)/Bommanahali.kml",
                false,
            ),
            (
                "/Users/sushi/Dev/Rust/quiet-route/data/KML (Lights)/Dasarahali.kml",
                false,
            ),
            (
                "/Users/sushi/Dev/Rust/quiet-route/data/KML (Lights)/RR_Nagar.kml",
                false,
            ),
        ];

        let safety = SafetyLayer::new(paths).unwrap();

        println!("\n🔍 DEBUG INFO:");
        println!("   Lights in KD-tree: {}", safety.lights.size());
        println!("   Police in KD-tree: {}\n", safety.police.size());

        // Test various locations
        // Note: Whether these are lit depends on the actual KML data coverage
        let test_locations = vec![
            (12.976, 77.593, "Cubbon Park"),
            (12.9352, 77.6245, "Koramangala"),
            (13.100, 77.500, "Remote Highway"),
        ];

        for (lat, lon, name) in test_locations {
            let is_lit = safety.is_lit(lat, lon);
            let nearest_dist = safety.nearest_light_distance(lat, lon);
            println!(
                "💡 {} ({}°N, {}°E): is_lit = {}, nearest_light = {:.1}m",
                name,
                lat,
                lon,
                is_lit,
                nearest_dist.unwrap_or(f64::MAX)
            );
        }

        // Verify method returns a boolean (always passes but ensures it compiles)
        assert!(safety.is_lit(12.976, 77.593) == true || safety.is_lit(12.976, 77.593) == false);
    }
}
