//! Linux sandbox helper entry point.
//!
//! On Linux, `codex-linux-sandbox` applies:
//! - in-process restrictions (`no_new_privs` + seccomp), and
//! - bubblewrap for filesystem isolation.
#[cfg(target_os = "linux")]
pub mod bazel_bwrap;
#[cfg(target_os = "linux")]
pub mod bundled_bwrap;
#[cfg(target_os = "linux")]
pub mod bwrap;
#[cfg(target_os = "linux")]
pub mod daemon_mounts;
#[cfg(target_os = "linux")]
pub mod exec_util;
#[cfg(target_os = "linux")]
pub mod fd_mount;
#[cfg(target_os = "linux")]
pub mod launcher;
#[cfg(target_os = "linux")]
pub mod linux_run_main;
#[cfg(target_os = "linux")]
pub mod proxy_lifecycle;
#[cfg(target_os = "linux")]
pub mod proxy_routing;
#[cfg(target_os = "linux")]
pub mod seccomp;
#[cfg(target_os = "linux")]
pub mod wslg;

#[cfg(target_os = "linux")]
pub use bundled_bwrap::find_bundled_bwrap_for_exe;
#[cfg(target_os = "linux")]
pub use bwrap::GLOB_SCAN_PROGRAM;
#[cfg(target_os = "linux")]
pub use bwrap::expand_unreadable_globs_in_environment;

/// Exit status returned when bundled bubblewrap fails digest verification.
#[cfg(target_os = "linux")]
pub const BUNDLED_BWRAP_DIGEST_VERIFICATION_FAILURE_EXIT_CODE: i32 = 8;

#[cfg(target_os = "linux")]
pub fn run_main() -> ! {
    linux_run_main::run_main();
}

#[cfg(not(target_os = "linux"))]
pub fn run_main() -> ! {
    panic!("codex-linux-sandbox is only supported on Linux");
}
