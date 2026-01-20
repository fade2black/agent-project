use std::time::{SystemTime, UNIX_EPOCH};

pub fn now() -> u64 {
    let start = SystemTime::now();
    let duration = start
        .duration_since(UNIX_EPOCH)
        .expect("Unable to get timestamp.");
    duration.as_secs()
}
