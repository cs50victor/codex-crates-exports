//! Extension crate for the `/goal` feature.

pub mod accounting;
pub mod analytics;
pub mod api;
pub mod events;
pub mod extension;
pub mod metrics;
pub mod runtime;
pub mod spec;
pub mod steering;
pub mod tool;

pub use api::GoalObjectiveUpdate;
pub use api::GoalService;
pub use api::GoalServiceError;
pub use api::GoalSetOutcome;
pub use api::GoalSetRequest;
pub use api::GoalTokenBudgetUpdate;
pub use extension::GoalExtension;
pub use extension::GoalExtensionConfig;
pub use extension::install_with_backend;
pub use runtime::GoalRuntimeHandle;
pub use runtime::PreviousGoalSnapshot;
pub use spec::CREATE_GOAL_TOOL_NAME;
pub use spec::GET_GOAL_TOOL_NAME;
pub use spec::UPDATE_GOAL_TOOL_NAME;
pub use tool::CreateGoalRequest;
