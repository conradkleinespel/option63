//! Configuration and validation for the CardDAV reverse proxy.

use crate::config::ConfigError;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;

pub const DEFAULT_PROXY_LISTEN: &str = "127.0.0.1:8080";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProxyConfig {
    #[serde(
        default,
        skip_serializing_if = "crate::config::serde_defaults::is_default"
    )]
    pub upstream: ProxyUpstreamConfig,
    #[serde(
        default,
        skip_serializing_if = "crate::config::serde_defaults::is_default"
    )]
    pub downstream: ProxyDownstreamConfig,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProxyUpstreamConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dangerous_allow_plain_text_http: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProxyDownstreamConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub listen: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls_cert_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls_key_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dangerous_trust_forwarded_proto: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dangerous_allow_acl_passthrough: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxyResolvedConfig {
    pub upstream: ProxyUpstreamResolvedConfig,
    pub downstream: ProxyDownstreamResolvedConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxyUpstreamResolvedConfig {
    pub url: reqwest::Url,
    pub username: String,
    pub password: String,
    pub dangerous_allow_plain_text_http: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxyDownstreamResolvedConfig {
    pub listen: SocketAddr,
    pub username: String,
    pub password: String,
    pub tls: Option<TlsSettings>,
    pub dangerous_trust_forwarded_proto: bool,
    pub dangerous_allow_acl_passthrough: bool,
}

impl ProxyConfig {
    pub fn resolve(&self) -> Result<ProxyResolvedConfig, ConfigError> {
        let ProxyConfig {
            upstream,
            downstream,
        } = self;
        let upstream = upstream
            .resolve()?
            .ok_or(ConfigError::MissingField("carddav.proxy.upstream"))?;
        let downstream = downstream
            .resolve()?
            .ok_or(ConfigError::MissingField("carddav.proxy.downstream"))?;
        Ok(ProxyResolvedConfig {
            upstream,
            downstream,
        })
    }
}

impl ProxyUpstreamConfig {
    pub fn resolve(&self) -> Result<Option<ProxyUpstreamResolvedConfig>, ConfigError> {
        let ProxyUpstreamConfig {
            url,
            username,
            password,
            dangerous_allow_plain_text_http,
        } = self;

        if url.is_none() && username.is_none() && password.is_none() {
            return Ok(None);
        }

        let url = url
            .as_deref()
            .ok_or(ConfigError::MissingField("carddav.proxy.upstream.url"))?;
        let username = username
            .as_deref()
            .ok_or(ConfigError::MissingField("carddav.proxy.upstream.username"))?;
        let password = password
            .as_deref()
            .ok_or(ConfigError::MissingField("carddav.proxy.upstream.password"))?;

        if password.is_empty() {
            return Err(ConfigError::EmptyPassword(
                "carddav.proxy.upstream.password".into(),
            ));
        }
        let allow_plain_text = dangerous_allow_plain_text_http.unwrap_or(false);
        if url.starts_with("http://") && !allow_plain_text {
            return Err(ConfigError::PlaintextUpstreamNotAllowed(url.to_string()));
        }
        let parsed: reqwest::Url =
            reqwest::Url::parse(url).map_err(|error| ConfigError::InvalidUrl {
                url: url.to_string(),
                reason: error.to_string(),
            })?;
        if !matches!(parsed.scheme(), "http" | "https") {
            return Err(ConfigError::UnsupportedUpstreamScheme(parsed.to_string()));
        }

        Ok(Some(ProxyUpstreamResolvedConfig {
            url: parsed,
            username: username.to_string(),
            password: password.to_string(),
            dangerous_allow_plain_text_http: allow_plain_text,
        }))
    }
}

impl ProxyDownstreamConfig {
    pub fn resolve(&self) -> Result<Option<ProxyDownstreamResolvedConfig>, ConfigError> {
        let ProxyDownstreamConfig {
            listen,
            username,
            password,
            tls_cert_path,
            tls_key_path,
            dangerous_trust_forwarded_proto,
            dangerous_allow_acl_passthrough,
        } = self;

        if username.is_none() && password.is_none() {
            return Ok(None);
        }

        let username = username.as_deref().ok_or(ConfigError::MissingField(
            "carddav.proxy.downstream.username",
        ))?;
        let password = password.as_deref().ok_or(ConfigError::MissingField(
            "carddav.proxy.downstream.password",
        ))?;
        if password.is_empty() {
            return Err(ConfigError::EmptyPassword(
                "carddav.proxy.downstream.password".into(),
            ));
        }

        let listen = match listen.as_deref() {
            Some(raw) => raw.to_string(),
            None => DEFAULT_PROXY_LISTEN.to_string(),
        };
        let listen: SocketAddr = listen
            .parse()
            .map_err(|error| ConfigError::InvalidListen(listen.clone(), error))?;

        let tls = TlsSettings::from_pair(
            tls_cert_path.as_deref().map(PathBuf::from),
            tls_key_path.as_deref().map(PathBuf::from),
        )?;

        let resolved = ProxyDownstreamResolvedConfig {
            listen,
            username: username.to_string(),
            password: password.to_string(),
            tls,
            dangerous_trust_forwarded_proto: dangerous_trust_forwarded_proto.unwrap_or(false),
            dangerous_allow_acl_passthrough: dangerous_allow_acl_passthrough.unwrap_or(false),
        };
        Self::validate(&resolved)?;
        Ok(Some(resolved))
    }

    fn validate(resolved: &ProxyDownstreamResolvedConfig) -> Result<(), ConfigError> {
        validate_listen(
            &resolved.listen,
            resolved.tls.is_some(),
            resolved.dangerous_trust_forwarded_proto,
        )
    }
}

/// Filesystem paths to the PEM-encoded TLS certificate chain and private key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlsSettings {
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
}

impl TlsSettings {
    pub fn from_pair(
        cert_path: Option<PathBuf>,
        key_path: Option<PathBuf>,
    ) -> Result<Option<Self>, ConfigError> {
        match (cert_path, key_path) {
            (Some(cert_path), Some(key_path)) => Ok(Some(Self {
                cert_path,
                key_path,
            })),
            (None, None) => Ok(None),
            _ => Err(ConfigError::InvalidTls(
                "tlsCertPath and tlsKeyPath must be set together or not at all".to_string(),
            )),
        }
    }
}

fn validate_listen(
    listen: &SocketAddr,
    tls_enabled: bool,
    trust_forwarded_proto: bool,
) -> Result<(), ConfigError> {
    if listen.ip().is_loopback() {
        return Ok(());
    }
    if tls_enabled || trust_forwarded_proto {
        return Ok(());
    }
    Err(ConfigError::NonLoopbackWithoutTls(*listen))
}
