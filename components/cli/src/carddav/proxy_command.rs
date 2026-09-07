use crate::error::CliError;
use actix_web::web;
use actix_web::{App, HttpServer};
use clap::ArgMatches;
use o63::carddav::config::ProxyConfig;
use o63::carddav::creds::SystemEnv;
use o63::carddav::error::ProxyError;
use o63::carddav::proxy::{
    AppState, catch_all, copy, delete, get, head, lock, mkcol, move_method, options, post,
    propfind, proppatch, put, report, unlock,
};
use reqwest::Client;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

fn method(name: &str) -> Result<actix_web::http::Method, ProxyError> {
    actix_web::http::Method::from_bytes(name.as_bytes())
        .map_err(|e| ProxyError::InvalidMethod(name.to_string(), e.to_string()))
}

pub fn handle_proxy_command(arg_matches: &ArgMatches) -> Result<(), CliError> {
    let config = ProxyConfig::new(
        arg_matches
            .get_one::<String>("upstream")
            .ok_or_else(|| CliError::InvalidInput("missing --upstream argument".to_string()))?,
        arg_matches
            .get_one::<String>("listen")
            .map(String::as_str)
            .unwrap_or("127.0.0.1:8080"),
        arg_matches.get_flag("dangerous-allow-plain-text-upstream"),
        arg_matches.get_flag("dangerous-log-request-response-body"),
        arg_matches.get_flag("verbose"),
        &SystemEnv,
    )?;

    actix_web::rt::System::new().block_on(run_proxy(config))?;
    Ok(())
}

async fn run_proxy(config: ProxyConfig) -> Result<(), ProxyError> {
    init_tracing(config.verbose);

    let state = Arc::new(AppState::new(
        config.upstream.clone(),
        Client::new(),
        config.credentials,
        config.dangerous_log_request_response_body,
    ));

    let m_propfind = method("PROPFIND")?;
    let m_proppatch = method("PROPPATCH")?;
    let m_mkcol = method("MKCOL")?;
    let m_get = method("GET")?;
    let m_head = method("HEAD")?;
    let m_post = method("POST")?;
    let m_put = method("PUT")?;
    let m_delete = method("DELETE")?;
    let m_copy = method("COPY")?;
    let m_move = method("MOVE")?;
    let m_lock = method("LOCK")?;
    let m_unlock = method("UNLOCK")?;
    let m_report = method("REPORT")?;
    let m_options = method("OPTIONS")?;

    tracing::info!(listen = %config.listen, upstream = %config.upstream, "carddav-proxy listening");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::from(state.clone()))
            .route(
                "/{tail:.*}",
                web::route().method(m_propfind.clone()).to(propfind),
            )
            .route(
                "/{tail:.*}",
                web::route().method(m_proppatch.clone()).to(proppatch),
            )
            .route("/{tail:.*}", web::route().method(m_mkcol.clone()).to(mkcol))
            .route("/{tail:.*}", web::route().method(m_get.clone()).to(get))
            .route("/{tail:.*}", web::route().method(m_head.clone()).to(head))
            .route("/{tail:.*}", web::route().method(m_post.clone()).to(post))
            .route("/{tail:.*}", web::route().method(m_put.clone()).to(put))
            .route(
                "/{tail:.*}",
                web::route().method(m_delete.clone()).to(delete),
            )
            .route("/{tail:.*}", web::route().method(m_copy.clone()).to(copy))
            .route(
                "/{tail:.*}",
                web::route().method(m_move.clone()).to(move_method),
            )
            .route("/{tail:.*}", web::route().method(m_lock.clone()).to(lock))
            .route(
                "/{tail:.*}",
                web::route().method(m_unlock.clone()).to(unlock),
            )
            .route(
                "/{tail:.*}",
                web::route().method(m_report.clone()).to(report),
            )
            .route(
                "/{tail:.*}",
                web::route().method(m_options.clone()).to(options),
            )
            .default_service(web::route().to(catch_all))
    })
    .bind(config.listen)
    .map_err(ProxyError::ServerStart)?
    .run()
    .await
    .map_err(ProxyError::ServerStart)
}

fn init_tracing(verbose: bool) {
    let default = if verbose { "debug" } else { "info" };
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
