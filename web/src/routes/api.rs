use actix_web::{HttpResponse, post, web};
use serde::{Deserialize, Serialize};
use vcard_lib::VCard;

#[derive(Deserialize)]
pub struct ParseRequest {
    pub vcard: String,
    #[serde(default)]
    pub props: Vec<String>,
}

#[derive(Serialize)]
pub struct ParseResponse {
    pub lines: Vec<ParsedLine>,
}

#[derive(Serialize)]
pub struct ParsedLine {
    pub name: String,
    pub value: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[post("/api/try/parse")]
pub async fn parse_vcard(req: web::Json<ParseRequest>) -> HttpResponse {
    let input = req.vcard.as_bytes();
    let allow_list: Vec<String> = req.props.iter().map(|p| p.to_ascii_uppercase()).collect();
    let has_filter = !allow_list.is_empty();

    let mut remaining = input;
    let mut lines = Vec::new();

    loop {
        let trimmed = trim_whitespace(remaining);
        if trimmed.is_empty() {
            break;
        }

        match VCard::parse(remaining, false) {
            Ok(out_vcard) => {
                for content_line in out_vcard.output().content_lines() {
                    let prop_name =
                        String::from_utf8_lossy(content_line.property().name().as_slice())
                            .to_ascii_uppercase();

                    let keep = if has_filter {
                        allow_list.contains(&prop_name)
                            || prop_name == "BEGIN"
                            || prop_name == "END"
                            || prop_name == "VERSION"
                    } else {
                        true
                    };

                    if keep {
                        let raw = String::from_utf8_lossy(content_line.to_vcard_vec().as_slice())
                            .to_string();
                        let value = extract_value(&raw);
                        lines.push(ParsedLine {
                            name: prop_name,
                            value,
                        });
                    }
                }
                remaining = out_vcard.remaining();
            }
            Err(err) => {
                return HttpResponse::BadRequest().json(ErrorResponse {
                    error: format!("Failed to parse vCard: {err:?}"),
                });
            }
        }
    }

    HttpResponse::Ok().json(ParseResponse { lines })
}

fn extract_value(raw: &str) -> String {
    if let Some(pos) = raw.find(':') {
        raw[pos + 1..]
            .trim_end_matches('\r')
            .trim_end_matches('\n')
            .to_string()
    } else {
        raw.trim().to_string()
    }
}

fn trim_whitespace(bytes: &[u8]) -> &[u8] {
    let start = bytes
        .iter()
        .position(|&b| !matches!(b, b'\r' | b'\n' | b' ' | b'\t'))
        .unwrap_or(bytes.len());

    let end = bytes
        .iter()
        .rev()
        .position(|&b| !matches!(b, b'\r' | b'\n' | b' ' | b'\t'))
        .map(|pos| bytes.len() - pos)
        .unwrap_or(bytes.len());

    &bytes[start..end]
}
