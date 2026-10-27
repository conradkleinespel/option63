use crate::carddav::common::ConnectionArgs;
use crate::error::CliError;
use clap::ArgMatches;

pub fn run(matches: &ArgMatches) -> Result<(), CliError> {
    let args = ConnectionArgs::from_matches(matches);
    let client = args.build_client()?;
    actix_web::rt::System::new().block_on(run_list(client))
}

async fn run_list(client: o63::carddav::client::CarddavClient) -> Result<(), CliError> {
    let principal = client.current_user_principal().await?.ok_or_else(|| {
        CliError::InvalidInput("server did not return a current-user-principal".into())
    })?;

    let home_sets = client.addressbook_home_set(&principal).await?;

    if home_sets.is_empty() {
        eprintln!("no addressbook home sets found");
        return Ok(());
    }

    for home_set in &home_sets {
        let addressbooks = client.find_addressbooks(home_set).await?;
        for addressbook in &addressbooks {
            match &addressbook.description {
                Some(desc) => println!("{}  {}", addressbook.href, desc),
                None => println!("{}", addressbook.href),
            }
        }
    }
    Ok(())
}
