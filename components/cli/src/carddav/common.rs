use crate::error::CliError;
use o63::carddav::client::CarddavClient;
use o63::config::ConfigError;
use std::path::PathBuf;
pub struct ConnectionArgs {
    pub config_files: Vec<PathBuf>,
}

impl ConnectionArgs {
    pub fn from_matches(matches: &clap::ArgMatches) -> Self {
        Self {
            config_files: matches
                .get_many::<String>("config")
                .map(|values| values.map(PathBuf::from).collect())
                .unwrap_or_default(),
        }
    }

    pub fn build_client(&self) -> Result<CarddavClient, CliError> {
        let config = o63::config::load(&self.config_files)?;
        let client = config
            .carddav
            .ok_or(ConfigError::MissingField("carddav"))?
            .client
            .ok_or(ConfigError::MissingField("carddav.client"))?
            .resolve()?;
        Ok(client.build_client())
    }
}

/// Join a collection and card name into a single card URL path, trimming a
/// trailing slash from the collection (matches `get`/`put`/`delete` behavior).
pub fn join_path(collection: &str, card: &str) -> String {
    let base = collection.trim_end_matches('/');
    format!("{base}/{card}")
}

pub fn add_connection_args(cmd: clap::Command) -> clap::Command {
    cmd.arg(
        clap::Arg::new("config")
            .long("config")
            .value_name("PATH")
            .help("JSON config file with carddav.client.url/username/password, can be specified multiple times")
            .required(false)
            .action(clap::ArgAction::Append),
    )
}
