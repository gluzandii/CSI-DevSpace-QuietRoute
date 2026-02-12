//! Geographic utility functions for distance calculations.
//!
//! Provides the Haversine formula implementation for calculating
//! great-circle distances between geographic coordinates on Earth.

/// Calculates the Haversine distance between two points in meters.
///
/// # Arguments
/// * `lat1` - Latitude of first point in degrees
/// * `lon1` - Longitude of first point in degrees
/// * `lat2` - Latitude of second point in degrees
/// * `lon2` - Longitude of second point in degrees
///
/// # Returns
/// Distance in meters
pub fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const EARTH_RADIUS_METERS: f64 = 6371000.0;

    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();

    let a =
        (dlat / 2.0).sin().powi(2) + lat1_rad.cos() * lat2_rad.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS_METERS * c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haversine_same_point() {
        let dist = haversine_distance(12.9, 77.5, 12.9, 77.5);
        assert!(dist.abs() < 0.1, "Same point should have nearly 0 distance");
    }

    #[test]
    fn test_haversine_one_degree_latitude() {
        // ~111km for 1 degree latitude
        let dist = haversine_distance(0.0, 0.0, 1.0, 0.0);
        assert!(
            dist > 110000.0 && dist < 112000.0,
            "1 degree latitude should be ~111km, got: {}",
            dist
        );
    }

    #[test]
    fn test_haversine_realistic_distance() {
        // From Cubbon Park to Electronic City in Bangalore (approximately 20km)
        let dist = haversine_distance(12.976, 77.593, 12.845, 77.663);
        assert!(
            dist > 15000.0 && dist < 25000.0,
            "Distance should be approximately 20km, got: {} meters",
            dist
        );
    }
}
