use crate::error::CliError;
use o63::carddav::creds;
use o63::carddav::creds::{PASSWORD_ENV, USERNAME_ENV};
use std::io::{self, Write};

pub fn handle_creds_command(_arg_matches: &clap::ArgMatches) -> Result<(), CliError> {
    let username = prompt("Username: ")?;
    let password = rpassword::prompt_password("Password: ")?;

    let username = username.trim().to_string();
    let (user, pass) = creds::encode(&username, &password);

    let stdout = io::stdout();
    write_creds(stdout.lock(), user, pass)?;
    Ok(())
}

fn write_creds(mut out: impl Write, user: String, pass: String) -> io::Result<()> {
    writeln!(out, "{USERNAME_ENV}={user}")?;
    writeln!(out, "{PASSWORD_ENV}={pass}")?;
    out.flush()
}

fn prompt(message: &str) -> io::Result<String> {
    let mut input = String::new();
    print!("{message}");
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;
    Ok(input)
}
