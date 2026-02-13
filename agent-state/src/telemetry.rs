use crate::Location;
use common::get_env_var;
use serde::Serialize;

pub fn energy() -> f64 {
    get_env_var("POWER_LEVEL")
}

pub fn location() -> Location {
    Location {
        lat: get_env_var("LAT"),
        lon: get_env_var("LON"),
    }
}

#[derive(Serialize)]
pub struct Telemetry {
    pub energy: f64,
    pub location: Location,
}

impl Telemetry {
    pub fn new() -> Self {
        Telemetry {
            energy: energy(),
            location: location(),
        }
    }
}
