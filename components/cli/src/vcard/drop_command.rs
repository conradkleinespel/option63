use crate::error::CliError;
use crate::io::{read_input, resolve_input, resolve_output, write_output};
use clap::ArgMatches;
use o63::vcard::VCard;
use regex::Regex;

pub fn handle_drop_command(arg_matches: &ArgMatches) -> Result<(), CliError> {
    let input = resolve_input(arg_matches.get_one::<String>("input").map(String::as_str))?;
    let field = arg_matches
        .get_one::<String>("field")
        .ok_or_else(|| CliError::InvalidInput("missing required 'field' argument".to_string()))?;
    let regex_pattern = arg_matches.get_one::<String>("regex");

    if !is_valid_property_name(field) {
        return Err(CliError::InvalidInput(format!(
            "Invalid field name: {field}. Must be a valid, droppable vCard property name."
        )));
    }

    let regex = match regex_pattern {
        Some(pattern) => Some(
            Regex::new(pattern)
                .map_err(|e| CliError::InvalidInput(format!("Invalid regex pattern: {e}")))?,
        ),
        None => None,
    };

    let content = read_input(&input)?;
    let rendered = render_drop(&content, field, regex.as_ref())?;
    let output = resolve_output(arg_matches.get_one::<String>("output").map(String::as_str));
    write_output(&output, &rendered)?;
    Ok(())
}

pub fn render_drop(
    content: &[u8],
    field: &str,
    regex: Option<&Regex>,
) -> Result<Vec<u8>, CliError> {
    let field_upper = field.to_ascii_uppercase();
    let mut output = Vec::new();

    let mut remaining = content;
    loop {
        if crate::trim_whitespace(remaining).is_empty() {
            break;
        }
        let (new_remaining, out_vcard) = VCard::parse(remaining, false)?;
        for content_line in out_vcard.content_lines() {
            let prop_name = String::from_utf8_lossy(content_line.property().name().as_slice())
                .to_ascii_uppercase();

            let should_keep = if prop_name == field_upper {
                match regex {
                    Some(re) => {
                        let value_bytes = content_line.property().value_to_vcard_vec();
                        let value = String::from_utf8_lossy(&value_bytes);
                        !re.is_match(value.as_ref())
                    }
                    None => false,
                }
            } else {
                true
            };

            if should_keep {
                output.extend_from_slice(content_line.to_vcard_vec().as_slice());
                output.extend_from_slice(b"\r\n");
            }
        }
        remaining = new_remaining;
    }

    Ok(output)
}

fn is_valid_property_name(field: &str) -> bool {
    let field_upper = field.to_ascii_uppercase();
    let bytes = field_upper.as_bytes();

    if bytes.starts_with(b"X-") {
        return true;
    }

    matches!(
        field_upper.as_str(),
        "ADR"
            | "ANNIVERSARY"
            | "BDAY"
            | "CALADRURI"
            | "CALURI"
            | "CATEGORIES"
            | "CLIENTPIDMAP"
            | "EMAIL"
            | "FBURL"
            | "FN"
            | "GENDER"
            | "GEO"
            | "IMPP"
            | "KEY"
            | "KIND"
            | "LANG"
            | "LOGO"
            | "MEMBER"
            | "NICKNAME"
            | "NOTE"
            | "N"
            | "ORG"
            | "PHOTO"
            | "PRODID"
            | "RELATED"
            | "REV"
            | "ROLE"
            | "SOUND"
            | "SOURCE"
            | "TEL"
            | "TITLE"
            | "TZ"
            | "UID"
            | "URL"
            | "XML"
    )
}
