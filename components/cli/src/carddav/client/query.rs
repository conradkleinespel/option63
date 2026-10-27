use crate::carddav::common::ConnectionArgs;
use crate::error::CliError;
use clap::ArgMatches;
use o63::carddav::protocol::{
    AddressDataProp, AddressDataRequest, AddressbookQuery, CARDDAV_NS, Filter, FilterTest,
    MatchType, PropFilter, PropFilterContent, TextMatch,
};
use o63::webdav::{Depth, Property, Propfind, QName};

pub fn run(matches: &ArgMatches) -> Result<(), CliError> {
    let args = ConnectionArgs::from_matches(matches);
    let client = args.build_client()?;
    let collection = matches
        .get_one::<String>("collection")
        .ok_or_else(|| CliError::InvalidInput("missing <collection> argument".into()))?
        .clone();
    let filter_prop = matches.get_one::<String>("filter-prop").cloned();
    let filter_value = matches.get_one::<String>("filter-value").cloned();
    let limit = match matches.get_one::<String>("limit") {
        Some(s) => match s.parse::<u32>() {
            Ok(n) => Some(n),
            Err(_) => {
                return Err(CliError::InvalidInput(format!(
                    "invalid --limit value `{s}`: expected positive integer"
                )));
            }
        },
        None => None,
    };
    actix_web::rt::System::new().block_on(run_query(
        client,
        &collection,
        filter_prop,
        filter_value,
        limit,
    ))
}

async fn run_query(
    client: o63::carddav::client::CarddavClient,
    collection: &str,
    filter_prop: Option<String>,
    filter_value: Option<String>,
    limit: Option<u32>,
) -> Result<(), CliError> {
    let prop_filters = match (filter_prop, filter_value) {
        (Some(prop), Some(value)) => vec![PropFilter {
            name: prop,
            test: FilterTest::AnyOf,
            content: PropFilterContent::Match {
                text_matches: vec![TextMatch {
                    collation: "i;unicode-casemap".into(),
                    negate_condition: false,
                    match_type: MatchType::Contains,
                    value,
                }],
                param_filters: vec![],
            },
        }],
        _ => vec![],
    };

    let query = AddressbookQuery {
        prop: Some(Propfind::Prop {
            properties: vec![Property::name_only(QName::dav("getetag"))],
        }),
        address_data: Some(AddressDataRequest {
            content_type: "text/vcard".into(),
            version: "3.0".into(),
            allprop: false,
            props: vec![
                AddressDataProp {
                    name: "FN".into(),
                    novalue: false,
                },
                AddressDataProp {
                    name: "UID".into(),
                    novalue: false,
                },
                AddressDataProp {
                    name: "EMAIL".into(),
                    novalue: false,
                },
            ],
        }),
        filter: Filter {
            test: FilterTest::AnyOf,
            prop_filters,
        },
        limit,
    };

    let multistatus = client
        .addressbook_query(collection, Depth::One, &query)
        .await?;

    for resp in &multistatus.responses {
        let card_data = resp
            .propstats()
            .and_then(|ps| ps.first())
            .and_then(|propstat| {
                propstat
                    .prop
                    .iter()
                    .find(|p| p.is(CARDDAV_NS, "address-data"))
                    .and_then(|p| match &p.value {
                        o63::webdav::PropertyValue::Text(t) => Some(t.as_str()),
                        _ => None,
                    })
            });
        match card_data {
            Some(data) => println!("{}", data),
            None => println!("{}", resp.href),
        }
    }
    Ok(())
}
