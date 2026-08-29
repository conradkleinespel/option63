use clap::ArgMatches;
use o63::VCard;
use o63::is_valid_property_name;
use regex::Regex;
use std::fs::File;
use std::io::{Error, Read, Write};

pub fn handle_remove_property_command(arg_matches: &ArgMatches) -> Result<(), Error> {
    let file_path = arg_matches.get_one::<String>("file").unwrap();
    let field = arg_matches.get_one::<String>("field").unwrap();
    let regex_pattern = arg_matches.get_one::<String>("regex");
    let output_file = arg_matches.get_one::<String>("output-file");

    if !is_valid_property_name(field) {
        return Err(Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "Invalid field name: {}. Must be a valid vCard property name.",
                field
            ),
        ));
    }

    let regex = if let Some(pattern) = regex_pattern {
        match Regex::new(pattern) {
            Ok(r) => Some(r),
            Err(e) => {
                return Err(Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Invalid regex pattern: {}", e),
                ));
            }
        }
    } else {
        None
    };

    let mut file = File::open(file_path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    let field_upper = field.to_ascii_uppercase();
    let mut output = Vec::new();

    let mut remaining = &content[..];
    loop {
        if crate::trim_whitespace(remaining).is_empty() {
            break;
        }
        match VCard::parse(remaining, false) {
            Ok(out_vcard) => {
                for content_line in out_vcard.output().content_lines() {
                    let prop_name =
                        String::from_utf8_lossy(content_line.property().name().as_slice())
                            .to_ascii_uppercase();

                    let should_keep = if prop_name == field_upper {
                        if let Some(ref re) = regex {
                            let value_bytes = content_line.property().to_vcard_vec();
                            let value = String::from_utf8_lossy(&value_bytes);
                            !re.is_match(value.as_ref())
                        } else {
                            false
                        }
                    } else {
                        true
                    };

                    if should_keep {
                        writeln!(
                            output,
                            "{}\r",
                            String::from_utf8_lossy(content_line.to_vcard_vec().as_slice())
                        )?;
                    }
                }
                remaining = out_vcard.remaining();
            }
            Err(err) => {
                eprintln!("failed to parse vcard: {:?}", err);
                break;
            }
        }
    }

    if let Some(output_path) = output_file {
        let mut out_file = File::create(output_path)?;
        out_file.write_all(&output)?;
    } else {
        print!("{}", String::from_utf8_lossy(&output));
    }

    Ok(())
}
