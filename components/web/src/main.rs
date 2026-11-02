mod redirect;
mod routes;

use actix_files::Files;
use actix_web::middleware::from_fn;
use actix_web::{App, HttpServer};
use clap::{Arg, ArgMatches, Command};
use o63::config::{Config, ConfigError, WebResolvedConfig};
use std::path::PathBuf;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let matches = parse_arg_matches();
    let resolved = load_web_config(&matches).map_err(to_io_error)?;
    let listen = resolved.listen;
    println!("listening on http://{listen}");

    HttpServer::new(move || {
        App::new()
            .wrap(from_fn(redirect::apex_to_www))
            .service(Files::new("/static", "./components/web/static").prefer_utf8(true))
            .configure(routes::configure)
    })
    .bind(listen)?
    .run()
    .await
}

fn parse_arg_matches() -> ArgMatches {
    Command::new("o63web")
        .version(env!("CARGO_PKG_VERSION"))
        .about("option63 website server")
        .arg(
            Arg::new("config")
                .long("config")
                .value_name("PATH")
                .help("JSON config file with the web role (repeatable, merged in order; must be owner-only, e.g. chmod 600)")
                .required(true)
                .action(clap::ArgAction::Append),
        )
        .get_matches()
}

fn load_web_config(matches: &ArgMatches) -> Result<WebResolvedConfig, ConfigError> {
    let files: Vec<PathBuf> = matches
        .get_many::<String>("config")
        .map(|values| values.map(PathBuf::from).collect())
        .unwrap_or_default();
    let config = o63::config::load(&files)?;
    resolve_web_config(config)
}

fn resolve_web_config(config: Config) -> Result<WebResolvedConfig, ConfigError> {
    config
        .web
        .ok_or(ConfigError::MissingField("web"))?
        .resolve()
}

fn to_io_error(error: ConfigError) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, error)
}
