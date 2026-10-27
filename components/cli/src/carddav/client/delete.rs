use crate::carddav::common::{ConnectionArgs, join_path};
use crate::error::CliError;
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
    actix_web::rt::System::new().block_on(run_delete(client, &collection, &card))
}

async fn run_delete(
    client: o63::carddav::client::CarddavClient,
    collection: &str,
    card: &str,
) -> Result<(), CliError> {
    let path = join_path(collection, card);
    let status = client.delete_card(&path).await?;
    eprintln!("DELETE {path} → {status}");
    Ok(())
}
