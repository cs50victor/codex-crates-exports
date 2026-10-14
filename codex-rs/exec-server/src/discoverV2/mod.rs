//! Executor-side V2 capability discovery.
//! Connection and RPC routing stay outside this module; legacy V1 scanning remains separate.

pub mod capability_discoveries;
pub mod capability_locations;
pub mod capability_manager;
