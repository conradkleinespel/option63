use crate::error::CliError;
use o63::config::prompt::{PromptIO, SystemPromptIO, prompt_optional, prompt_password, prompt_yn};
use o63::config::{
    CarddavConfig, ClientConfig, Config, ProxyConfig, ProxyDownstreamConfig, ProxyUpstreamConfig,
};
use std::io::Write;
use std::path::Path;

/// Interactive config author and editor: `o63 carddav config [--client]
/// [--proxy] --output FILE [--force]`. The sole interactive surface of the CLI.
/// Prompts every field of the selected role(s); on an existing `--output` it
/// seeds the prompts from that file and, with `--force`, writes the result back
/// in place. Never writes a config to stdout.
pub fn run(arg_matches: &clap::ArgMatches) -> Result<(), CliError> {
    let include_client = arg_matches.get_flag("client");
    let include_proxy = arg_matches.get_flag("proxy");
    if !include_client && !include_proxy {
        return Err(CliError::InvalidInput(
            "carddav config requires at least one of --client or --proxy".to_string(),
        ));
    }
    let output = match arg_matches.get_one::<String>("output") {
        Some(path) => Path::new(path).to_path_buf(),
        None => {
            return Err(CliError::InvalidInput(
                "carddav config requires --output FILE (no default; the config is never written to stdout)".to_string(),
            ))
        }
    };
    let force = arg_matches.get_flag("force");

    let seed = if output.exists() {
        if !force {
            return Err(CliError::InvalidInput(format!(
                "refusing to overwrite existing file `{}`; pass --force to edit it in place",
                output.display()
            )));
        }
        Some(o63::config::load(std::slice::from_ref(&output))?)
    } else {
        None
    };

    let mut io = SystemPromptIO;
    let doc = build_config_document(include_client, include_proxy, seed.as_ref(), &mut io)?;

    write_config_file(&output, &doc, force)
}

pub use run as handle_config_command;

/// Require a value for a field where empty input is not acceptable (create mode
/// with no current value). `None` means the operator provided nothing.
fn require_value(field: &str, value: Option<String>) -> Result<String, CliError> {
    value.ok_or_else(|| CliError::InvalidInput(format!("{field} must not be empty")))
}

fn build_config_document(
    include_client: bool,
    include_proxy: bool,
    seed: Option<&Config>,
    io: &mut dyn PromptIO,
) -> Result<String, CliError> {
    let root_default = seed
        .and_then(|c| c.carddav.as_ref())
        .and_then(|c| c.dangerous_log_request_response_body)
        .unwrap_or(false);
    let log_body = prompt_yn(
        io,
        "Log request and response bodies (dangerousLogRequestResponseBody)",
        root_default,
    )
    .map_err(CliError::Io)?;

    let mut document = Config {
        carddav: Some(CarddavConfig {
            dangerous_log_request_response_body: Some(log_body),
            ..CarddavConfig::default()
        }),
    };

    if include_client {
        let current = seed
            .and_then(|c| c.carddav.as_ref())
            .and_then(|c| c.client.clone());
        document.carddav.as_mut().unwrap().client = Some(prompt_client(current, io)?);
    }

    if include_proxy {
        let current = seed
            .and_then(|c| c.carddav.as_ref())
            .and_then(|c| c.proxy.clone());
        document.carddav.as_mut().unwrap().proxy = Some(prompt_proxy(current, io)?);
    }

    // On edit, roles not selected are preserved untouched from the seed file.
    if !include_client
        && let Some(seed_config) = seed
        && let Some(client) = seed_config.carddav.as_ref().and_then(|c| c.client.clone())
        && client != ClientConfig::default()
    {
        document.carddav.as_mut().unwrap().client = Some(client);
    }
    if !include_proxy
        && let Some(seed_config) = seed
        && let Some(proxy) = seed_config.carddav.as_ref().and_then(|c| c.proxy.clone())
        && proxy != ProxyConfig::default()
    {
        document.carddav.as_mut().unwrap().proxy = Some(proxy);
    }

    Ok(serde_json::to_string_pretty(&document)
        .map_err(|e| CliError::InvalidInput(format!("failed to serialize config: {e}")))?
        + "\n")
}

fn prompt_client(
    current: Option<ClientConfig>,
    io: &mut dyn PromptIO,
) -> Result<ClientConfig, CliError> {
    io.heading("Client (carddav.client) configuration:\n")
        .map_err(CliError::Io)?;
    let url = require_value(
        "client url",
        prompt_optional(
            io,
            "Server URL",
            current.as_ref().and_then(|c| c.url.as_deref()),
        )
        .map_err(CliError::Io)?,
    )?;
    let username = require_value(
        "client username",
        prompt_optional(
            io,
            "Username",
            current.as_ref().and_then(|c| c.username.as_deref()),
        )
        .map_err(CliError::Io)?,
    )?;
    let password = require_value(
        "client password",
        prompt_password(
            io,
            "Password",
            current.as_ref().and_then(|c| c.password.as_deref()),
        )
        .map_err(CliError::Io)?,
    )?;
    let allow_plain_text_http = prompt_yn(
        io,
        "Allow plain-text HTTP",
        current
            .as_ref()
            .is_some_and(|c| c.dangerous_allow_plain_text_http.unwrap_or(false)),
    )
    .map_err(CliError::Io)?;

    Ok(ClientConfig {
        url: Some(url),
        username: Some(username),
        password: Some(password),
        dangerous_allow_plain_text_http: Some(allow_plain_text_http),
    })
}

fn prompt_proxy(
    current: Option<ProxyConfig>,
    io: &mut dyn PromptIO,
) -> Result<ProxyConfig, CliError> {
    io.heading("Proxy (carddav.proxy) configuration:\n")
        .map_err(CliError::Io)?;
    io.heading("  Upstream server (carddav.proxy.upstream):\n")
        .map_err(CliError::Io)?;
    let upstream = current.as_ref().map(|c| &c.upstream);
    let upstream_url = require_value(
        "proxy upstream url",
        prompt_optional(
            io,
            "    Server URL",
            upstream.and_then(|c| c.url.as_deref()),
        )
        .map_err(CliError::Io)?,
    )?;
    let upstream_username = require_value(
        "proxy upstream username",
        prompt_optional(
            io,
            "    Username",
            upstream.and_then(|c| c.username.as_deref()),
        )
        .map_err(CliError::Io)?,
    )?;
    let upstream_password = require_value(
        "proxy upstream password",
        prompt_password(
            io,
            "    Password",
            upstream.and_then(|c| c.password.as_deref()),
        )
        .map_err(CliError::Io)?,
    )?;
    let upstream_allow_plain_text_http = prompt_yn(
        io,
        "    Allow plain-text HTTP",
        upstream.is_some_and(|c| c.dangerous_allow_plain_text_http.unwrap_or(false)),
    )
    .map_err(CliError::Io)?;

    io.heading("  Downstream (carddav.proxy.downstream):\n")
        .map_err(CliError::Io)?;
    let downstream = current.as_ref().map(|c| &c.downstream);
    let current_listen = downstream
        .and_then(|c| c.listen.as_deref())
        .unwrap_or(o63::config::DEFAULT_PROXY_LISTEN);
    let listen = prompt_optional(io, "    Listen", Some(current_listen))
        .map_err(CliError::Io)?
        .unwrap_or_else(|| o63::config::DEFAULT_PROXY_LISTEN.to_string());
    let downstream_username = require_value(
        "proxy downstream username",
        prompt_optional(
            io,
            "    Proxy username",
            downstream.and_then(|c| c.username.as_deref()),
        )
        .map_err(CliError::Io)?,
    )?;
    let downstream_password = require_value(
        "proxy downstream password",
        prompt_password(
            io,
            "    Proxy password",
            downstream.and_then(|c| c.password.as_deref()),
        )
        .map_err(CliError::Io)?,
    )?;
    let tls_cert_path = prompt_optional(
        io,
        "    TLS certificate path (tlsCertPath)",
        downstream.and_then(|c| c.tls_cert_path.as_deref()),
    )
    .map_err(CliError::Io)?;
    let tls_key_path = prompt_optional(
        io,
        "    TLS private key path (tlsKeyPath)",
        downstream.and_then(|c| c.tls_key_path.as_deref()),
    )
    .map_err(CliError::Io)?;
    let trust_forwarded_proto = prompt_yn(
        io,
        "    Trust forwarded proto (dangerousTrustForwardedProto)",
        downstream.is_some_and(|c| c.dangerous_trust_forwarded_proto.unwrap_or(false)),
    )
    .map_err(CliError::Io)?;
    let allow_acl_passthrough = prompt_yn(
        io,
        "    Allow ACL passthrough (dangerousAllowAclPassthrough)",
        downstream.is_some_and(|c| c.dangerous_allow_acl_passthrough.unwrap_or(false)),
    )
    .map_err(CliError::Io)?;

    Ok(ProxyConfig {
        upstream: ProxyUpstreamConfig {
            url: Some(upstream_url),
            username: Some(upstream_username),
            password: Some(upstream_password),
            dangerous_allow_plain_text_http: Some(upstream_allow_plain_text_http),
        },
        downstream: ProxyDownstreamConfig {
            listen: Some(listen),
            username: Some(downstream_username),
            password: Some(downstream_password),
            tls_cert_path,
            tls_key_path,
            dangerous_trust_forwarded_proto: Some(trust_forwarded_proto),
            dangerous_allow_acl_passthrough: Some(allow_acl_passthrough),
        },
    })
}

/// Write the config to `path` as owner-only. Refuses to overwrite an existing
/// file unless `force` is set (the caller already validated this for edits).
fn write_config_file(path: &Path, document: &str, force: bool) -> Result<(), CliError> {
    let mut options = std::fs::OpenOptions::new();
    if force {
        options.create(true).truncate(true).write(true);
    } else {
        options.create_new(true).write(true);
    }
    let mut file = options.open(path)?;
    file.write_all(document.as_bytes())?;
    file.flush()?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}
