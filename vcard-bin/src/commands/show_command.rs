use crate::trim_whitespace;
use clap::ArgMatches;
use std::fs;
use std::io::{Error, Read};
use vcard_lib::VCard;

pub fn handle_show_command(arg_matches: &ArgMatches) -> Result<(), Error> {
    let file_path = arg_matches.get_one::<String>("file").unwrap();

    let props_to_display: Option<Vec<String>> = arg_matches
        .get_many::<String>("props")
        .map(|vals| vals.cloned().collect());

    let strict = arg_matches.get_flag("strict");

    let mut file = fs::File::open(file_path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    let mut remaining = &content[..];
    loop {
        if trim_whitespace(remaining).is_empty() {
            break;
        }
        match VCard::parse(remaining, strict) {
            Ok(out_vcard) => {
                for content_line in out_vcard.output().content_lines() {
                    let prop_name =
                        String::from_utf8_lossy(content_line.property().name().as_slice())
                            .to_ascii_uppercase();
                    let should_display = match &props_to_display {
                        Some(props) => {
                            props.contains(&prop_name)
                                || prop_name == "BEGIN"
                                || prop_name == "END"
                                || prop_name == "VERSION"
                        }
                        None => true,
                    };

                    if should_display {
                        // Do print \r, to make diffs with existing .vcf files easier
                        println!(
                            "{}\r",
                            String::from_utf8_lossy(content_line.to_vcard_vec().as_slice())
                        );
                    }
                }
                remaining = out_vcard.remaining();
            }
            Err(err) => {
                println!("failed to parse vcard: {:?}", err,);
                break;
            }
        }
    }
    Ok(())
}
