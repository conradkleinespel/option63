pub mod carddav;
pub mod error;
pub mod io;
pub mod vcard;

use crate::carddav::creds_command::handle_creds_command;
use crate::carddav::proxy_command::handle_proxy_command;
use crate::error::CliError;
use crate::vcard::drop_command::handle_drop_command;
use crate::vcard::show_command::handle_show_command;
use clap::{Arg, ArgMatches, Command};
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), CliError> {
    let matches = parse_arg_matches();
    match matches.subcommand() {
        Some(("vcard", vcard_matches)) => match vcard_matches.subcommand() {
            Some(("show", show_arg_matches)) => handle_show_command(show_arg_matches)?,
            Some(("drop", drop_arg_matches)) => handle_drop_command(drop_arg_matches)?,
            _ => unreachable!("Exhausted list of vcard subcommands and subcommand_required(true)"),
        },
        Some(("carddav", carddav_matches)) => match carddav_matches.subcommand() {
            Some(("creds", creds_arg_matches)) => handle_creds_command(creds_arg_matches)?,
            Some(("proxy", proxy_arg_matches)) => handle_proxy_command(proxy_arg_matches)?,
            _ => {
                unreachable!("Exhausted list of carddav subcommands and subcommand_required(true)")
            }
        },
        _ => unreachable!("Exhausted list of subcommands and subcommand_required(true)"),
    }
    Ok(())
}

fn parse_arg_matches() -> ArgMatches {
    Command::new("option63")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Conrad Kleinespel")
        .about("option63 helps you manage vCards")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("vcard")
                .about("vCard inspection and transformation commands")
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("show")
                        .about("Show vCard properties")
                        .arg(
                            Arg::new("input")
                                .help("Path to the vCard file, or '-' to read from stdin")
                                .required(true)
                                .index(1),
                        )
                        .arg(
                            Arg::new("props")
                                .short('p')
                                .long("props")
                                .help("Comma-separated list of properties to display")
                                .required(false)
                                .value_delimiter(','),
                        )
                        .arg(
                            Arg::new("strict")
                                .short('s')
                                .long("strict")
                                .help("Enable strict RFC parsing mode")
                                .required(false)
                                .action(clap::ArgAction::SetTrue),
                        )
                        .arg(
                            Arg::new("output")
                                .long("output")
                                .help("Output file path, or '-' for stdout (default)")
                                .required(false),
                        ),
                )
                .subcommand(
                    Command::new("drop")
                        .about("Drop vCard property fields matching optional regex")
                        .arg(
                            Arg::new("input")
                                .help("Path to the vCard file, or '-' to read from stdin")
                                .required(true)
                                .index(1),
                        )
                        .arg(
                            Arg::new("field")
                                .help("Property name to drop (e.g., TEL, EMAIL)")
                                .required(true)
                                .index(2),
                        )
                        .arg(
                            Arg::new("regex")
                                .long("regex")
                                .help("Optional regex pattern - only drop if value matches")
                                .required(false),
                        )
                        .arg(
                            Arg::new("strict")
                                .short('s')
                                .long("strict")
                                .help("Enable strict RFC parsing mode")
                                .required(false)
                                .action(clap::ArgAction::SetTrue),
                        )
                        .arg(
                            Arg::new("output")
                                .long("output")
                                .help("Output file path, or '-' for stdout (default)")
                                .required(false),
                        ),
                ),
        )
        .subcommand(
            Command::new("carddav")
                .about("CardDAV commands")
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("creds").about(
                        "Print base64-encoded upstream credentials as O63_CARDDAV_PROXY_* variables",
                    ),
                )
                .subcommand(
                    Command::new("proxy")
                        .about("Run a CardDAV reverse proxy that forwards traffic to an upstream server")
                        .arg(
                            Arg::new("listen")
                                .short('l')
                                .long("listen")
                                .help("Address to listen on, e.g. 127.0.0.1:8080")
                                .default_value("127.0.0.1:8080")
                                .required(false),
                        )
                        .arg(
                            Arg::new("verbose")
                                .short('v')
                                .long("verbose")
                                .help("Enable DEBUG logging level")
                                .required(false)
                                .action(clap::ArgAction::SetTrue),
                        )
                        .arg(
                            Arg::new("upstream")
                                .short('u')
                                .long("upstream")
                                .help("Upstream CardDAV server URL, e.g. https://dav.example.com")
                                .required(true),
                        )
                        .arg(
                            Arg::new("dangerous-log-request-response-body")
                                .long("dangerous-log-request-response-body")
                                .help("Log full request/response bodies")
                                .required(false)
                                .action(clap::ArgAction::SetTrue),
                        )
                        .arg(
                            Arg::new("dangerous-allow-plain-text-upstream")
                                .long("dangerous-allow-plain-text-upstream")
                                .help("Allow the upstream server to use plain-text HTTP instead of HTTPS")
                                .required(false)
                                .action(clap::ArgAction::SetTrue),
                        ),
                ),
        )
        .get_matches()
}

fn trim_whitespace(bytes: &[u8]) -> &[u8] {
    let start = bytes
        .iter()
        .position(|&b| !matches!(b, b'\r' | b'\n' | b' ' | b'\t'))
        .unwrap_or(bytes.len());

    let end = bytes
        .iter()
        .rev()
        .position(|&b| !matches!(b, b'\r' | b'\n' | b' ' | b'\t'))
        .map(|pos| bytes.len() - pos)
        .unwrap_or(bytes.len());

    &bytes[start..end]
}
