#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos as platform;

#[cfg(target_os = "linux")]
mod macos;
#[cfg(target_os = "linux")]
use macos as platform;

pub fn now() -> u64 {
    platform::now()
}
