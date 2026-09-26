pub mod aliases;
pub mod catalog;
pub mod catalog_prompt;
pub mod config;
pub mod dynamic_skill_selector;
pub mod extension;
pub mod fragments;
pub mod host_aliases;
pub mod host_outcome;
pub mod host_prompt;
pub mod host_roots;
pub mod host_service;
pub mod host_snapshot;
pub mod invocation;
pub mod loader;
pub mod provider;
pub mod render;
pub mod render_observability;
pub mod selection;
pub mod shadow_selection_experiment;
pub mod skills_extension_state;
pub mod sources;
pub mod state;
pub mod telemetry;
pub mod tools;
pub mod warnings;
pub mod world_state;
pub mod world_state_catalogs;

pub use config::SkillsExtensionConfig;
pub use extension::install;
pub use extension::install_with_providers;
pub use extension::install_with_providers_and_metrics;
pub use host_outcome::SkillLoadOutcome;
pub use host_prompt::HostSkillPrompts;
pub use host_prompt::InjectedHostSkillPrompts;
pub use host_service::HostSkillsLoadInput;
pub use host_service::HostSkillsRequest;
pub use host_service::HostSkillsService;
pub use host_snapshot::HostSkillsSnapshot;
pub use invocation::detect_implicit_skill_invocation;
pub use provider::ExecutorSkillProvider;
pub use provider::HostSkillProvider;
pub use provider::SkillProvider;
pub(crate) use skills_extension_state::SkillsExtensionState;
pub use sources::SkillProviderSource;
pub use sources::SkillProviders;
pub use state::SkillsThreadState;
pub use telemetry::record_plugin_turn_usage;

/// Recognizes persisted explicit skill prompts without exposing their fragment implementation.
pub fn is_skill_prompt_fragment(text: &str) -> bool {
    <fragments::SkillInstructions as codex_extension_api::ContextualUserFragment>::matches_text(
        text,
    )
}
