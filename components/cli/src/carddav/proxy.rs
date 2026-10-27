use crate::error::CliError;
use actix_web::{App, HttpServer};
use clap::ArgMatches;
use o63::carddav::client::CarddavClient;
use o63::carddav::error::ProxyError;
use o63::carddav::server_handler_proxy::ProxyHandler;
use o63::config::permission::reject_loose_permissions;
use o63::config::{ConfigError, TlsSettings};
use o63::webdav::actix_server::dav_service;
use o63::webdav::auth::BasicAuth;
use o63::webdav::client::{Credentials, DavClient};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

pub fn run(arg_matches: &ArgMatches) -> Result<(), CliError> {
    let config_files: Vec<PathBuf> = arg_matches
        .get_many::<String>("config")
        .map(|values| values.map(PathBuf::from).collect())
        .unwrap_or_default();
    let config = o63::config::load(&config_files)?;
    let resolved = config
        .carddav
        .ok_or(ConfigError::MissingField("carddav"))?
        .resolve()?;

    let proxy = resolved
        .proxy
        .ok_or(ConfigError::MissingField("carddav.proxy.upstream.url"))?;

    let upstream_credentials = Credentials {
        username: proxy.upstream.username.clone(),
        password: proxy.upstream.password.clone(),
    };
    let downstream_credentials = Credentials {
        username: proxy.downstream.username.clone(),
        password: proxy.downstream.password.clone(),
    };

    let server_config = ProxyServerConfig {
        listen: proxy.downstream.listen,
        upstream: proxy.upstream.url,
        upstream_credentials,
        downstream_credentials,
        dangerous_allow_acl_passthrough: proxy.downstream.dangerous_allow_acl_passthrough,
        tls: proxy.downstream.tls,
    };

    actix_web::rt::System::new().block_on(run_proxy(server_config))?;
    Ok(())
}

/// Assembled server context for `carddav proxy`, built from the resolved proxy
/// config. The config is already validated by resolve, including the TLS
/// cert/key both-or-neither rule.
struct ProxyServerConfig {
    listen: SocketAddr,
    upstream: reqwest::Url,
    upstream_credentials: Credentials,
    downstream_credentials: Credentials,
    dangerous_allow_acl_passthrough: bool,
    tls: Option<TlsSettings>,
}

async fn run_proxy(config: ProxyServerConfig) -> Result<(), ProxyError> {
    ::tracing::info!(
        listen = %config.listen,
        upstream = %config.upstream,
        "carddav-proxy listening"
    );

    let dav = DavClient::new(config.upstream).with_credentials(config.upstream_credentials);
    let client = CarddavClient::new(dav);
    let handler = ProxyHandler::new_with_acl(client, config.dangerous_allow_acl_passthrough);

    let auth = BasicAuth::new(config.downstream_credentials.clone());

    let server = HttpServer::new(move || {
        App::new()
            .wrap(auth.clone())
            .service(dav_service(handler.clone()))
    });

    let server = match &config.tls {
        Some(tls) => {
            let server_config = load_tls_config(&tls.cert_path, &tls.key_path)?;
            server
                .bind_rustls_0_23(config.listen, server_config)
                .map_err(ProxyError::ServerStart)?
        }
        None => server
            .bind(config.listen)
            .map_err(ProxyError::ServerStart)?,
    };

    server.run().await.map_err(ProxyError::ServerStart)
}

/// Load a rustls server configuration from PEM certificate and key files.
/// The key file must be owner-only, mirroring `--config` enforcement.
fn load_tls_config(cert_path: &Path, key_path: &Path) -> Result<rustls::ServerConfig, ProxyError> {
    reject_loose_permissions(key_path)
        .map_err(|e| ProxyError::TlsLoad(format!("TLS key file check failed: {e}").into()))?;
    use rustls::pki_types::pem::PemObject;
    use rustls::pki_types::{CertificateDer, PrivateKeyDer};

    let cert_iter =
        CertificateDer::pem_file_iter(cert_path).map_err(|e| ProxyError::TlsLoad(Box::new(e)))?;
    let certs: Vec<CertificateDer<'static>> = cert_iter
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| ProxyError::TlsLoad(Box::new(e)))?;

    let key =
        PrivateKeyDer::from_pem_file(key_path).map_err(|e| ProxyError::TlsLoad(Box::new(e)))?;

    rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| ProxyError::TlsLoad(Box::new(e)))
}
