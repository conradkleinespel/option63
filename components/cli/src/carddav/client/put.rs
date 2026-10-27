use crate::carddav::common::{ConnectionArgs, join_path};
use crate::error::CliError;
use crate::io::{read_input, resolve_input};
use clap::ArgMatches;
use o63::vcard::VCard;

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
    let input = matches
        .get_one::<String>("input")
        .ok_or_else(|| CliError::InvalidInput("missing <input> argument".into()))?
        .clone();
    actix_web::rt::System::new().block_on(run_put(client, &collection, &card, &input))
}

async fn run_put(
    client: o63::carddav::client::CarddavClient,
    collection: &str,
    card: &str,
    input: &str,
) -> Result<(), CliError> {
    let input_src = resolve_input(Some(input))?;
    let body = read_input(&input_src)?;
    validate_card_body(&body)?;

    let path = join_path(collection, card);
    let status = client.put_card(&path, &body).await?;

    if status == 200 || status == 201 {
        eprintln!("PUT {path} → {status}");
    } else {
        eprintln!("PUT {path} → {status} (unexpected)");
    }
    Ok(())
}

/// Reject a PUT body that is not a valid vCard before sending it to the server.
fn validate_card_body(body: &[u8]) -> Result<(), CliError> {
    VCard::parse(body, false)?;
    Ok(())
}
