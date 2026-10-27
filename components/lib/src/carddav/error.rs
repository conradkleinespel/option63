//! Error types for CardDAV servers and clients.
//!
//! Runtime-only errors for actually running the proxy. Configuration-level
//! validation errors live in [`crate::config::ConfigError`].

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProxyError {
    #[error("failed to load TLS certificate/key: {0}")]
    TlsLoad(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("failed to start HTTP server: {0}")]
    ServerStart(#[source] std::io::Error),
    #[error("upstream request failed: {0}")]
    Upstream(#[source] reqwest::Error),
    #[error("invalid HTTP method `{0}`: {1}")]
    InvalidMethod(String, String),
    #[error("failed to serialize WebDAV error XML response: {0}")]
    XmlSerialize(#[source] std::io::Error),
}
