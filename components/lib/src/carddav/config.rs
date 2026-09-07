use crate::carddav::creds;
use crate::carddav::creds::Credentials;
use crate::carddav::error::ProxyError;
use reqwest::Url;
use std::net::SocketAddr;

#[derive(Debug)]
pub struct ProxyConfig {
    pub listen: SocketAddr,
    pub upstream: Url,
    pub credentials: Credentials,
    pub dangerous_log_request_response_body: bool,
    pub verbose: bool,
}

impl ProxyConfig {
    pub fn new(
        upstream: &str,
        listen: &str,
        dangerous_allow_plain_text_upstream: bool,
        dangerous_log_request_response_body: bool,
        verbose: bool,
        env: &dyn creds::EnvReader,
    ) -> Result<Self, ProxyError> {
        let listen: SocketAddr = listen
            .parse()
            .map_err(|e| ProxyError::InvalidListen(listen.to_string(), e))?;

        let upstream = Url::parse(upstream)
            .map_err(|e| ProxyError::InvalidUpstream(upstream.to_string(), e.to_string()))?;

        if !matches!(upstream.scheme(), "http" | "https") {
            return Err(ProxyError::UnsupportedUpstreamScheme(upstream.to_string()));
        }

        if upstream.scheme() == "http" && !dangerous_allow_plain_text_upstream {
            return Err(ProxyError::PlaintextUpstreamNotAllowed(
                upstream.to_string(),
            ));
        }

        let credentials = creds::load_credentials(env)?;

        Ok(Self {
            listen,
            upstream,
            credentials,
            dangerous_log_request_response_body,
            verbose,
        })
    }
}
