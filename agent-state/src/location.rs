use common::approx_equal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Location {
    pub lat: f64,
    pub lon: f64,
}

impl Location {
    pub fn new(lat: f64, lon: f64) -> Self {
        Location { lat, lon }
    }

    pub fn distance_to(&self, other: &Location) -> f64 {
        let dlat = (self.lat - other.lat) * 111.0;
        let dlon = (self.lon - other.lon) * 111.0 * self.lat.to_radians().cos();

        // return in meters
        1000.0 * (dlat.powi(2) + dlon.powi(2)).sqrt()
    }
}

impl PartialEq for Location {
    fn eq(&self, other: &Self) -> bool {
        approx_equal(self.lat, other.lat) && approx_equal(self.lon, other.lon)
    }
}
