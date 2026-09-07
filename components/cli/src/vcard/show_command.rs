use crate::error::CliError;
use crate::io::{read_input, resolve_input, resolve_output, write_output};
use crate::trim_whitespace;
use clap::ArgMatches;
use o63::vcard::VCard;

pub fn handle_show_command(arg_matches: &ArgMatches) -> Result<(), CliError> {
    let input = resolve_input(arg_matches.get_one::<String>("input").map(String::as_str))?;
    let content = read_input(&input)?;

    let props_to_display: Option<Vec<String>> = arg_matches
        .get_many::<String>("props")
        .map(|vals| vals.cloned().collect());

    let strict = arg_matches.get_flag("strict");

    let rendered = render_show(&content, props_to_display.as_deref(), strict)?;
    let output = resolve_output(arg_matches.get_one::<String>("output").map(String::as_str));
    write_output(&output, &rendered)?;
    Ok(())
}

pub fn render_show(
    content: &[u8],
    props_to_display: Option<&[String]>,
    strict: bool,
) -> Result<Vec<u8>, CliError> {
    let mut output = Vec::new();
    let mut remaining = content;
    loop {
        if trim_whitespace(remaining).is_empty() {
            break;
        }
        let (new_remaining, out_vcard) = VCard::parse(remaining, strict)?;
        for content_line in out_vcard.content_lines() {
            let prop_name = String::from_utf8_lossy(content_line.property().name().as_slice())
                .to_ascii_uppercase();
            let should_display = match props_to_display {
                Some(props) => {
                    props.contains(&prop_name)
                        || prop_name == "BEGIN"
                        || prop_name == "END"
                        || prop_name == "VERSION"
                }
                None => true,
            };

            if should_display {
                // Do write \r\n, to make diffs with existing .vcf files easier
                output.extend_from_slice(content_line.to_vcard_vec().as_slice());
                output.extend_from_slice(b"\r\n");
            }
        }
        remaining = new_remaining;
    }
    Ok(output)
}
