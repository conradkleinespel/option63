//! Client configuration for the direct CardDAV commands.

use crate::carddav::client::CarddavClient;
use crate::config::ConfigError;
use crate::webdav::client::{Credentials, DavClient};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClientConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dangerous_allow_plain_text_http: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientResolvedConfig {
    pub url: reqwest::Url,
    pub username: String,
    pub password: String,
    pub dangerous_allow_plain_text_http: bool,
}

impl ClientConfig {
    /// Resolve the client role. Returns `Ok(None)` when the role carries no
    /// required field at all; errors when a required field is present but
    /// another is missing (naming it); otherwise a fully validated resolved
    /// value with defaults applied.
    pub fn resolve(&self) -> Result<ClientResolvedConfig, ConfigError> {
        let ClientConfig {
            url,
            username,
            password,
            dangerous_allow_plain_text_http,
        } = self;

        let url = url
            .as_deref()
            .ok_or(ConfigError::MissingField("carddav.client.url"))?;
        let username = username
            .as_deref()
            .ok_or(ConfigError::MissingField("carddav.client.username"))?;
        let password = password
            .as_deref()
            .ok_or(ConfigError::MissingField("carddav.client.password"))?;

        if password.is_empty() {
            return Err(ConfigError::EmptyPassword("carddav.client.password".into()));
        }
        let allow_plain_text = dangerous_allow_plain_text_http.unwrap_or(false);
        if url.starts_with("http://") && !allow_plain_text {
            return Err(ConfigError::PlaintextClientNotAllowed {
                url: url.to_string(),
            });
        }
        let url = parse_client_url(url)?;

        Ok(ClientResolvedConfig {
            url,
            username: username.to_string(),
            password: password.to_string(),
            dangerous_allow_plain_text_http: allow_plain_text,
        })
    }
}

impl ClientResolvedConfig {
    pub fn build_client(&self) -> CarddavClient {
        let credentials = Credentials {
            username: self.username.clone(),
            password: self.password.clone(),
        };
        let dav = DavClient::new(self.url.clone()).with_credentials(credentials);
        CarddavClient::new(dav)
    }
}

fn parse_client_url(value: &str) -> Result<reqwest::Url, ConfigError> {
    let parsed = reqwest::Url::parse(value).map_err(|error| ConfigError::InvalidUrl {
        url: value.to_string(),
        reason: error.to_string(),
    })?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(ConfigError::UnsupportedClientScheme(parsed.to_string()));
    }
    Ok(parsed)
}
