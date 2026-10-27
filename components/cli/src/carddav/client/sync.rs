use crate::carddav::common::ConnectionArgs;
use crate::error::CliError;
use clap::ArgMatches;
use o63::webdav::{Property, QName, SyncCollection, SyncLevel};

pub fn run(matches: &ArgMatches) -> Result<(), CliError> {
    let args = ConnectionArgs::from_matches(matches);
    let client = args.build_client()?;
    let collection = matches
        .get_one::<String>("collection")
        .ok_or_else(|| CliError::InvalidInput("missing <collection> argument".into()))?
        .clone();
    let token = matches.get_one::<String>("sync-token").cloned();
    actix_web::rt::System::new().block_on(run_sync(client, &collection, token))
}

async fn run_sync(
    client: o63::carddav::client::CarddavClient,
    collection: &str,
    token: Option<String>,
) -> Result<(), CliError> {
    let sync_collection = SyncCollection {
        sync_token: token,
        sync_level: SyncLevel::One,
        prop: vec![Property::name_only(QName::dav("getetag"))],
        limit: None,
    };

    let multistatus = client.sync_collection(collection, &sync_collection).await?;

    for resp in &multistatus.responses {
        let status = resp
            .propstats()
            .and_then(|ps| ps.first())
            .map(|propstat| propstat.status.reason.clone())
            .or_else(|| resp.status().map(|s| s.reason.clone()))
            .unwrap_or_default();
        if status.contains("200") {
            println!("{}", resp.href);
        } else {
            println!("{}  {}", resp.href, status);
        }
    }

    if let Some(t) = &multistatus.sync_token {
        eprintln!("sync-token: {t}");
    }
    Ok(())
}
