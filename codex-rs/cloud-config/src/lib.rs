//! Cloud-hosted configuration data for Codex.
//!
//! This crate owns transport, caching, and refresh behavior for cloud-delivered
//! config data. Parsing and composition remain in `codex-config`.

pub mod backend;
pub mod bundle_loader;
pub mod cache;
pub mod metrics;
pub mod service;
pub mod validation;

pub use bundle_loader::cloud_config_bundle_loader;
pub use bundle_loader::cloud_config_bundle_loader_for_storage;
pub use bundle_loader::cloud_config_bundle_loader_for_storage_without_cache;
