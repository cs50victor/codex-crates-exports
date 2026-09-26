pub mod chatgpt_cloudflare_cookies;
pub mod chatgpt_hosts;
pub mod client;
pub mod client_builder;
pub mod client_tls;
pub mod custom_ca;
pub mod error;
pub mod network_policy;
pub mod outbound_proxy;
pub mod request;
pub mod request_builder;
pub mod request_draft;
pub mod response;
pub mod retry_after;
pub mod route_aware_client_pool;
pub mod route_aware_redirect;
pub mod tls_backend_fallback;
pub mod transport;

pub use crate::chatgpt_cloudflare_cookies::with_chatgpt_cloudflare_cookie_store;
pub use crate::chatgpt_hosts::is_allowed_chatgpt_host;
pub use crate::client::HttpClient;
pub use crate::client::HttpError;
pub use crate::client_builder::HttpClientBuilder;
pub use crate::client_tls::HttpClientTlsConfig;
pub use crate::custom_ca::BuildCustomCaTransportError;
/// Test-only subprocess hook for custom CA coverage.
///
/// This stays public only so the `custom_ca_probe` binary target can reuse the shared helper. It
/// is hidden from normal docs because ordinary callers should use
/// [`build_reqwest_client_with_custom_ca`] instead.
#[doc(hidden)]
pub use crate::custom_ca::build_reqwest_client_for_subprocess_tests;
pub use crate::custom_ca::build_reqwest_client_with_custom_ca;
pub use crate::custom_ca::build_rustls_client_config_with_custom_ca;
pub use crate::custom_ca::maybe_build_rustls_client_config_with_custom_ca;
pub use crate::error::StreamError;
pub use crate::error::TransportError;
pub use crate::network_policy::DestinationPolicy;
pub use crate::network_policy::NetworkPermit;
pub use crate::network_policy::NetworkPolicy;
pub use crate::network_policy::NetworkPolicyController;
pub use crate::network_policy::NetworkPolicyDenied;
pub use crate::network_policy::NetworkPolicyRevision;
pub use crate::outbound_proxy::BuildRouteAwareHttpClientError;
pub use crate::outbound_proxy::ClientRouteClass;
pub use crate::outbound_proxy::HttpClientFactory;
#[cfg(target_os = "macos")]
pub use crate::outbound_proxy::MacosSystemProxyConfiguration;
pub use crate::outbound_proxy::OutboundProxyPolicy;
pub use crate::outbound_proxy::OutboundProxyRoute;
pub use crate::outbound_proxy::RouteFailureClass;
#[doc(hidden)]
pub use crate::outbound_proxy::cache_system_proxy_route_for_test;
#[cfg(target_os = "macos")]
pub use crate::outbound_proxy::macos_system_proxy_configuration;
pub use crate::request::EncodedJsonBody;
pub use crate::request::PreparedRequestBody;
pub use crate::request::Request;
pub use crate::request::RequestBody;
pub use crate::request::RequestCompression;
pub use crate::request::Response;
pub use crate::request_builder::RequestBuilder;
pub use crate::response::HttpResponse;
pub use crate::retry_after::RetryAfter;
pub use crate::route_aware_client_pool::RouteAwareClientPool;
pub use crate::route_aware_client_pool::RouteAwareClientPoolError;
pub use crate::route_aware_client_pool::RouteAwareRequestError;
pub use crate::transport::ByteStream;
pub use crate::transport::HttpTransport;
pub use crate::transport::ReqwestTransport;
pub use crate::transport::StreamResponse;

#[cfg(windows)]
pub mod windows_tls;

#[cfg(windows)]
pub use crate::windows_tls::build_windows_platform_tls_config;
