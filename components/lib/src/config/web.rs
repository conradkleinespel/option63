//! Configuration for the option63 website server.

use crate::config::ConfigError;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

pub const DEFAULT_WEB_LISTEN: &str = "0.0.0.0:8080";
pub const DEFAULT_DATABASE_PORT: u16 = 5432;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WebConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub listen: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "crate::config::serde_defaults::is_default"
    )]
    pub database: WebDatabaseConfig,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WebDatabaseConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "crate::config::serde_defaults::is_default"
    )]
    pub tls: WebDatabaseTlsConfig,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WebDatabaseTlsConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebResolvedConfig {
    pub listen: SocketAddr,
    pub database: WebDatabaseResolvedConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebDatabaseResolvedConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub username: String,
    pub password: String,
    pub tls_enabled: bool,
}

impl WebConfig {
    pub fn resolve(&self) -> Result<WebResolvedConfig, ConfigError> {
        let WebConfig { listen, database } = self;
        let listen = match listen.as_deref() {
            Some(raw) => raw.to_string(),
            None => DEFAULT_WEB_LISTEN.to_string(),
        };
        let listen: SocketAddr = listen
            .parse()
            .map_err(|error| ConfigError::InvalidListen(listen.clone(), error))?;
        Ok(WebResolvedConfig {
            listen,
            database: database.resolve()?,
        })
    }
}

impl WebDatabaseConfig {
    pub fn resolve(&self) -> Result<WebDatabaseResolvedConfig, ConfigError> {
        let WebDatabaseConfig {
            host,
            port,
            name,
            username,
            password,
            tls,
        } = self;

        let host = host
            .as_deref()
            .ok_or(ConfigError::MissingField("web.database.host"))?;
        let name = name
            .as_deref()
            .ok_or(ConfigError::MissingField("web.database.name"))?;
        let username = username
            .as_deref()
            .ok_or(ConfigError::MissingField("web.database.username"))?;
        let password = password
            .as_deref()
            .ok_or(ConfigError::MissingField("web.database.password"))?;

        if password.is_empty() {
            return Err(ConfigError::EmptyPassword("web.database.password".into()));
        }

        Ok(WebDatabaseResolvedConfig {
            host: host.to_string(),
            port: port.unwrap_or(DEFAULT_DATABASE_PORT),
            name: name.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            tls_enabled: tls.enabled.unwrap_or(false),
        })
    }
}
