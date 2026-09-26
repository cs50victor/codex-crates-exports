pub mod agent_resolver;
pub mod api;
pub mod child_config;
pub mod control;
pub mod registry;
pub mod role;
pub mod status;
pub mod types;

pub(crate) use codex_protocol::protocol::AgentStatus;
pub(crate) use control::LocalAgentControl;
pub(crate) use registry::exceeds_thread_spawn_depth_limit;
pub(crate) use registry::next_thread_spawn_depth;
pub(crate) use status::agent_status_from_event;
