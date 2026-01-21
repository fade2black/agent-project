pub mod serialization;
pub use serialization::*;
pub mod time;
use std::env;

pub fn get_env_var<T: std::str::FromStr>(name: &str) -> T {
    if let Ok(val) = env::var(name) {
        if let Ok(val) = val.parse::<T>() {
            val
        } else {
            panic!("Failed to parse env var {}", name);
        }
    } else {
        panic!("Missing env var {}", name);
    }
}
