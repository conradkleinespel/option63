use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProxyError {
    #[error("invalid listen address `{0}`: {1}")]
    InvalidListen(String, #[source] std::net::AddrParseError),
    #[error("invalid upstream URL `{0}`: {1}")]
    InvalidUpstream(String, String),
    #[error("unsupported upstream URL scheme in `{0}`, expected http or https")]
    UnsupportedUpstreamScheme(String),
    #[error(
        "upstream URL `{0}` uses plain-text HTTP, which is not allowed; pass --dangerous-allow-plain-text-upstream to override"
    )]
    PlaintextUpstreamNotAllowed(String),
    #[error("failed to start HTTP server: {0}")]
    ServerStart(#[source] std::io::Error),
    #[error("upstream request failed: {0}")]
    Upstream(#[source] reqwest::Error),
    #[error("invalid HTTP method `{0}`: {1}")]
    InvalidMethod(String, String),
    #[error("missing or empty environment variable `{0}` for upstream credentials")]
    CredsMissingEnvironment(String),
    #[error("failed to base64-decode credentials field `{0}` in environment variable `{1}`: {2}")]
    CredsBase64Decode(String, String, #[source] base64::DecodeError),
    #[error("malformed WWW-Authenticate challenge: {0}")]
    InvalidChallenge(String),
    #[error("failed to serialize WebDAV error XML response: {0}")]
    XmlSerialize(#[source] std::io::Error),
}
