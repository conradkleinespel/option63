//! Configuration file data models, JSON deserialization, and resolution.

pub mod client;
pub mod merge;
pub mod permission;
pub mod prompt;
pub mod proxy;

use crate::config::permission::reject_loose_permissions;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub use client::{ClientConfig, ClientResolvedConfig};
pub use merge::merge;
pub use proxy::{
    DEFAULT_PROXY_LISTEN, ProxyConfig, ProxyDownstreamConfig, ProxyDownstreamResolvedConfig,
    ProxyResolvedConfig, ProxyUpstreamConfig, ProxyUpstreamResolvedConfig, TlsSettings,
};

mod serde_defaults {
    pub fn is_default<T: Default + PartialEq>(value: &T) -> bool {
        value == &T::default()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub carddav: Option<CarddavConfig>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CarddavConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client: Option<ClientConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxy: Option<ProxyConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dangerous_log_request_response_body: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CarddavResolvedConfig {
    pub client: Option<ClientResolvedConfig>,
    pub proxy: Option<ProxyResolvedConfig>,
    pub dangerous_log_request_response_body: bool,
}

impl CarddavConfig {
    pub fn resolve(&self) -> Result<CarddavResolvedConfig, ConfigError> {
        let CarddavConfig {
            client,
            proxy,
            dangerous_log_request_response_body,
        } = self;
        Ok(CarddavResolvedConfig {
            client: match client {
                Some(c) => Some(c.resolve()?),
                None => None,
            },
            proxy: match proxy {
                Some(c) => Some(c.resolve()?),
                None => None,
            },
            dangerous_log_request_response_body: dangerous_log_request_response_body
                .unwrap_or(false),
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("io error while reading config `{path}`: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse config file `{path}` as JSON: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("config file `{0}` is not owner-only, run `chmod 600 {0}`")]
    InvalidPermissions(PathBuf),
    #[error("`{0}` must not be empty")]
    EmptyPassword(String),
    #[error("failed to read input during interactive prompt: {0}")]
    Prompt(#[source] std::io::Error),
    #[error("missing field `{0}`")]
    MissingField(&'static str),
    #[error("invalid CardDAV URL `{url}`: {reason}")]
    InvalidUrl { url: String, reason: String },
    #[error("cannot call plain-text HTTP CardDAV server `{url}`")]
    PlaintextClientNotAllowed { url: String },
    #[error("unsupported CardDAV client URL scheme in `{0}`, expected http or https")]
    UnsupportedClientScheme(String),
    #[error("invalid listen address `{0}`: {1}")]
    InvalidListen(String, #[source] std::net::AddrParseError),
    #[error("unsupported upstream URL scheme in `{0}`, expected http or https")]
    UnsupportedUpstreamScheme(String),
    #[error("cannot user plain-text HTTP upstream URL `{0}`")]
    PlaintextUpstreamNotAllowed(String),
    #[error("cannot bind non-loopback address `{0}` without TLS")]
    NonLoopbackWithoutTls(std::net::SocketAddr),
    #[error("invalid TLS configuration: {0}")]
    InvalidTls(String),
}

pub fn load(files: &[PathBuf]) -> Result<Config, ConfigError> {
    let mut acc = Config::default();
    for path in files {
        reject_loose_permissions(path)?;
        let doc = parse_document(path)?;
        acc = merge(acc, doc);
    }
    Ok(acc)
}

fn parse_document(path: &Path) -> Result<Config, ConfigError> {
    let raw = std::fs::read_to_string(path).map_err(|e| ConfigError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    serde_json::from_str(&raw).map_err(|e| ConfigError::Parse {
        path: path.to_path_buf(),
        source: e,
    })
}
