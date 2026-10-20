//! Ephemeral OpenAI runtimes: composite registration and per-request assertions.

pub mod actor;
pub mod auth;
pub mod bootstrap;
pub mod manager;
pub mod registration;
pub mod runtime;

pub use actor::AgentHost;
pub use actor::AuthenticatedActor;
pub use bootstrap::ActorInfo;
pub use bootstrap::AgentHostAuthorization;
pub use bootstrap::AuthenticatedActorAuthorization;
pub use bootstrap::RuntimeRegistrationMetadata;
pub use bootstrap::RuntimeRegistrationRequest;
pub use bootstrap::RuntimeRegistrationResponse;
pub use bootstrap::RuntimeRole;

pub use auth::RuntimeAuth;
pub use manager::RuntimeAuthError;
pub use manager::RuntimeAuthManager;
pub use registration::DEFAULT_REGISTRATION_URL;
pub use registration::RegistrationError;
pub use registration::RegistrationErrorCode;
pub use registration::RuntimeRegistrationClient;
pub use runtime::RuntimeAgentInstance;
pub use runtime::RuntimeRegistration;
