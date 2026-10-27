use crate::carddav::common::{ConnectionArgs, join_path};
use crate::error::CliError;
use crate::io::{resolve_output, write_output};
use clap::ArgMatches;

pub fn run(matches: &ArgMatches) -> Result<(), CliError> {
    let args = ConnectionArgs::from_matches(matches);
    let client = args.build_client()?;
    let collection = matches
        .get_one::<String>("collection")
        .ok_or_else(|| CliError::InvalidInput("missing <collection> argument".into()))?
        .clone();
    let card = matches
        .get_one::<String>("card")
        .ok_or_else(|| CliError::InvalidInput("missing <card> argument".into()))?
        .clone();
    let output = matches.get_one::<String>("output").cloned();
    actix_web::rt::System::new().block_on(run_get(client, &collection, &card, output))
}

async fn run_get(
    client: o63::carddav::client::CarddavClient,
    collection: &str,
    card: &str,
    output: Option<String>,
) -> Result<(), CliError> {
    let path = join_path(collection, card);
    let response = client.get_card(&path).await?;

    let out = resolve_output(output.as_deref());
    write_output(&out, &response.body)?;
    Ok(())
}
