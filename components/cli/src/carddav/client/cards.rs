use crate::carddav::common::ConnectionArgs;
use crate::error::CliError;
use clap::ArgMatches;
use o63::carddav::protocol::CARDDAV_NS;
use o63::webdav::{Depth, Property, PropertyValue, Propfind, QName};

pub fn run(matches: &ArgMatches) -> Result<(), CliError> {
    let args = ConnectionArgs::from_matches(matches);
    let client = args.build_client()?;
    let collection = matches
        .get_one::<String>("collection")
        .ok_or_else(|| CliError::InvalidInput("missing <collection> argument".into()))?
        .clone();
    actix_web::rt::System::new().block_on(run_cards(client, &collection))
}

async fn run_cards(
    client: o63::carddav::client::CarddavClient,
    collection: &str,
) -> Result<(), CliError> {
    let propfind = Propfind::Prop {
        properties: vec![
            Property::name_only(QName::dav("displayname")),
            Property::name_only(QName::dav("getetag")),
            Property::name_only(QName::new(CARDDAV_NS, "address-data")),
        ],
    };
    let multistatus = client
        .dav()
        .propfind(collection, Depth::One, &propfind)
        .await?;

    for resp in &multistatus.responses {
        if resp.href.trim_end_matches('/') == collection.trim_end_matches('/') {
            continue;
        }
        let etag = resp
            .propstats()
            .and_then(|ps| ps.first())
            .and_then(|propstat| {
                propstat
                    .prop
                    .iter()
                    .find(|p| p.is(o63::webdav::xml::DAV_NS, "getetag"))
                    .and_then(|p| match &p.value {
                        PropertyValue::Text(t) => Some(t.clone()),
                        _ => None,
                    })
            });
        match &etag {
            Some(e) => println!("{}  {}", resp.href, e),
            None => println!("{}", resp.href),
        }
    }
    Ok(())
}
