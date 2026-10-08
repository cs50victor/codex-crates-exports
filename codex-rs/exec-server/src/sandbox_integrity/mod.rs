#[cfg_attr(target_os = "linux", path = "bubblewrap.rs")]
#[cfg_attr(target_os = "macos", path = "seatbelt.rs")]
#[cfg_attr(target_os = "windows", path = "mxc.rs")]
pub mod backend;
pub mod dependencies;
pub mod runner;
pub mod telemetry;

pub use runner::run_integrity_checks;

#[cfg(test)]
#[path = "backend_tests.rs"]
mod backend_tests;
