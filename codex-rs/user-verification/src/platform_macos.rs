//! macOS Security and LocalAuthentication integration.

pub mod error;
#[cfg(target_os = "macos")]
pub mod key_protection;
#[cfg(target_os = "macos")]
pub mod provider;

#[cfg(target_os = "macos")]
pub(crate) use provider::NativeProvider;
#[cfg(target_os = "macos")]
pub(crate) use provider::device_supported;
