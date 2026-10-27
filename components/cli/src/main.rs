pub mod carddav;
pub mod error;
pub mod io;
pub mod tracing;
pub mod vcard;

use crate::error::CliError;
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

fn is_verbose(matches: &ArgMatches) -> bool {
    if matches.get_flag("verbose") {
        return true;
    }
    if let Some((_, sub)) = matches.subcommand() {
        return is_verbose(sub);
    }
    false
}

fn run() -> Result<(), CliError> {
    let matches = parse_arg_matches();
    let verbose = is_verbose(&matches);
    tracing::init(verbose);
    match matches.subcommand() {
        Some(("vcard", vcard_matches)) => match vcard_matches.subcommand() {
            Some(("show", show_arg_matches)) => vcard::show::run(show_arg_matches)?,
            Some(("drop", drop_arg_matches)) => vcard::drop::run(drop_arg_matches)?,
            _ => unreachable!("Exhausted list of vcard subcommands and subcommand_required(true)"),
        },
        Some(("carddav", carddav_matches)) => match carddav_matches.subcommand() {
            Some(("config", config_arg_matches)) => carddav::config::run(config_arg_matches)?,
            Some(("proxy", proxy_arg_matches)) => carddav::proxy::run(proxy_arg_matches)?,
            Some(("client", client_matches)) => match client_matches.subcommand() {
                Some(("list", sub_matches)) => carddav::client::list::run(sub_matches)?,
                Some(("cards", sub_matches)) => carddav::client::cards::run(sub_matches)?,
                Some(("query", sub_matches)) => carddav::client::query::run(sub_matches)?,
                Some(("get", sub_matches)) => carddav::client::get::run(sub_matches)?,
                Some(("put", sub_matches)) => carddav::client::put::run(sub_matches)?,
                Some(("delete", sub_matches)) => carddav::client::delete::run(sub_matches)?,
                Some(("sync", sub_matches)) => carddav::client::sync::run(sub_matches)?,
                _ => {
                    unreachable!(
                        "Exhausted list of carddav client subcommands and subcommand_required(true)"
                    )
                }
            },
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
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable DEBUG level logging via the tracing integration")
                .global(true)
                .action(clap::ArgAction::SetTrue),
        )
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
                    Command::new("config")
                        .about("Interactively create or edit an owner-only config file for the client and/or proxy")
                        .arg(
                            Arg::new("client")
                                .long("client")
                                .help("Write the carddav.client object for the direct commands (list, cards, query, get, put, delete, sync)")
                                .required(false)
                                .action(clap::ArgAction::SetTrue),
                        )
                        .arg(
                            Arg::new("proxy")
                                .long("proxy")
                                .help("Write the carddav.proxy object (upstream + downstream) for carddav proxy")
                                .required(false)
                                .action(clap::ArgAction::SetTrue),
                        )
                        .arg(
                            Arg::new("output")
                                .long("output")
                                .value_name("PATH")
                                .help("Output config file path (required; never stdout)")
                                .required(true),
                        )
                        .arg(
                            Arg::new("force")
                                .long("force")
                                .help("Edit the output file in place if it already exists, seeding prompts from its values")
                                .required(false)
                                .action(clap::ArgAction::SetTrue),
                        )
                        .group(
                            clap::ArgGroup::new("role")
                                .required(true)
                                .multiple(true)
                                .args(["client", "proxy"]),
                        ),
                )
                .subcommand(
                    Command::new("proxy")
                        .about("Run a CardDAV reverse proxy that forwards traffic to an upstream server")
                        .arg(
                            Arg::new("config")
                                .long("config")
                                .value_name("PATH")
                                .help("JSON config file with carddav.proxy.upstream.url/username/password and carddav.proxy.downstream.listen/username/password, carddav.dangerousLogRequestResponseBody (repeatable, merged in order; must be owner-only, e.g. chmod 600; generate one with `o63 carddav config --proxy`)")
                                .required(false)
                                .action(clap::ArgAction::Append),
                        ),
                )
                .subcommand(
                    Command::new("client")
                        .about("CardDAV client commands")
                        .subcommand_required(true)
                        .arg_required_else_help(true)
                        .subcommand(
                            crate::carddav::common::add_connection_args(
                                Command::new("list")
                                    .about("Discover and list address books on a CardDAV server")
                            )
                        )
                        .subcommand(
                            crate::carddav::common::add_connection_args(
                                Command::new("cards")
                                    .about("List cards in an address book collection")
                                    .arg(
                                        Arg::new("collection")
                                            .help("Address book collection path, e.g. /addressbooks/user/contacts/")
                                            .required(true)
                                            .index(1),
                                    )
                            )
                        )
                        .subcommand(
                            crate::carddav::common::add_connection_args(
                                Command::new("query")
                                    .about("Send an addressbook-query report to find matching cards")
                                    .arg(
                                        Arg::new("collection")
                                            .help("Address book collection path")
                                            .required(true)
                                            .index(1),
                                    )
                                    .arg(
                                        Arg::new("filter-prop")
                                            .long("filter-prop")
                                            .help("vCard property name to filter on (e.g. FN, EMAIL)")
                                            .required(false),
                                    )
                                    .arg(
                                        Arg::new("filter-value")
                                            .long("filter-value")
                                            .help("Value to match (contains match)")
                                            .required(false),
                                    )
                                    .arg(
                                        Arg::new("limit")
                                            .long("limit")
                                            .help("Maximum number of results")
                                            .required(false),
                                    )
                            )
                        )
                        .subcommand(
                            crate::carddav::common::add_connection_args(
                                Command::new("get")
                                    .about("Get a vCard from an address book")
                                    .arg(
                                        Arg::new("collection")
                                            .help("Address book collection path")
                                            .required(true)
                                            .index(1),
                                    )
                                    .arg(
                                        Arg::new("card")
                                            .help("Card filename, e.g. cyrus.vcf")
                                            .required(true)
                                            .index(2),
                                    )
                                    .arg(
                                        Arg::new("output")
                                            .short('o')
                                            .long("output")
                                            .help("Output file path (default: stdout)")
                                            .required(false),
                                    )
                            )
                        )
                        .subcommand(
                            crate::carddav::common::add_connection_args(
                                Command::new("put")
                                    .about("Upload a vCard to an address book")
                                    .arg(
                                        Arg::new("collection")
                                            .help("Address book collection path")
                                            .required(true)
                                            .index(1),
                                    )
                                    .arg(
                                        Arg::new("card")
                                            .help("Card filename, e.g. cyrus.vcf")
                                            .required(true)
                                            .index(2),
                                    )
                                    .arg(
                                        Arg::new("input")
                                            .help("Input file path, or '-' for stdin")
                                            .required(true)
                                            .index(3),
                                    )
                            )
                        )
                        .subcommand(
                            crate::carddav::common::add_connection_args(
                                Command::new("delete")
                                    .about("Delete a vCard from an address book")
                                    .arg(
                                        Arg::new("collection")
                                            .help("Address book collection path")
                                            .required(true)
                                            .index(1),
                                    )
                                    .arg(
                                        Arg::new("card")
                                            .help("Card filename, e.g. cyrus.vcf")
                                            .required(true)
                                            .index(2),
                                    )
                            )
                        )
                        .subcommand(
                            crate::carddav::common::add_connection_args(
                                Command::new("sync")
                                    .about("Synchronize with an address book using sync-collection")
                                    .arg(
                                        Arg::new("collection")
                                            .help("Address book collection path")
                                            .required(true)
                                            .index(1),
                                    )
                                    .arg(
                                        Arg::new("sync-token")
                                            .long("sync-token")
                                            .help("Previous sync token (omit for initial sync)")
                                            .required(false),
                                    )
                            )
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
