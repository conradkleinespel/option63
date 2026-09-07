use crate::carddav::creds::Credentials;
use crate::carddav::error::ProxyError;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use nom::IResult;
use nom::Parser;
use nom::branch::alt;
use nom::bytes::complete::{escaped_transform, take_while1};
use nom::character::complete::{anychar, char, multispace0, none_of};
use nom::combinator::map;
use nom::multi::separated_list0;
use nom::sequence::delimited;

#[derive(Debug, Clone)]
pub struct Challenge {
    pub scheme: String,
}

pub fn basic_header(credentials: &Credentials) -> String {
    let combined = format!("{}:{}", credentials.username, credentials.password);
    format!("Basic {}", BASE64.encode(combined.as_bytes()))
}

pub fn parse_www_authenticate(header: &str) -> Result<Challenge, ProxyError> {
    let challenges = parse_challenges(header)?;

    let challenge = challenges
        .iter()
        .find(|c| c.scheme.eq_ignore_ascii_case("basic"))
        .ok_or_else(|| {
            ProxyError::InvalidChallenge(format!("no `basic` challenge in `{header}`"))
        })?;

    Ok(Challenge {
        scheme: challenge.scheme.clone(),
    })
}

/// Parse a `WWW-Authenticate` header into all of its challenges, regardless of
/// scheme. Per RFC 9110 (§11.2) a field is `[ challenge *( OWS "," OWS
/// challenge ) ]`.
fn parse_challenges(header: &str) -> Result<Vec<ParsedChallenge>, ProxyError> {
    let (rest, challenges) = challenges(header)
        .map_err(|e| ProxyError::InvalidChallenge(format!("failed to parse `{header}`: {e}")))?;

    if !rest.trim().is_empty() {
        return Err(ProxyError::InvalidChallenge(format!(
            "trailing data after challenge in `{header}`: `{rest}`"
        )));
    }

    Ok(challenges)
}

/// Per RFC 9110 (§11.2), a `WWW-Authenticate` field is `[ challenge *( OWS "," OWS
/// challenge ) ]`, where each challenge is
/// `auth-scheme [ 1*SP ( token68 / [ auth-param *( OWS "," OWS auth-param ) ] ) ]`.
///
/// We only support the `#auth-param` form, so we parse a token `auth-scheme`
/// followed by any number of comma-separated `key=value` parameters. Values may
/// be quoted strings (with backslash escapes) or bare tokens.
#[derive(Debug, Clone)]
struct ParsedChallenge {
    scheme: String,
    /// Parsed `auth-param`s (realm, nonce, ...). Kept primarily for test
    /// assertions; `basic` (the only supported scheme) does not use them.
    #[allow(dead_code)]
    params: std::collections::HashMap<String, String>,
}

fn challenges(i: &str) -> IResult<&str, Vec<ParsedChallenge>> {
    let (i, _) = multispace0(i)?;
    separated_list0(
        (multispace0, char(','), multispace0),
        delimited(multispace0, challenge, multispace0),
    )
    .parse(i)
}

fn challenge(i: &str) -> IResult<&str, ParsedChallenge> {
    let (i, scheme) = map(token, |s: &str| s.to_string()).parse(i)?;
    // The parameters carry the realm/nonce/etc, but only `basic` is supported,
    // and Basic ignores them. We still parse them so the header structure is
    // validated (a malformed challenge is rejected).
    let (i, params) = auth_params(i)?;
    Ok((i, ParsedChallenge { scheme, params }))
}

fn auth_params(i: &str) -> IResult<&str, std::collections::HashMap<String, String>> {
    let (i, _) = multispace0(i)?;
    let (i, params) = separated_list0(
        (multispace0, char(','), multispace0),
        delimited(multispace0, auth_param, multispace0),
    )
    .parse(i)?;
    Ok((i, params.into_iter().collect()))
}

fn auth_param(i: &str) -> IResult<&str, (String, String)> {
    // auth-param = token BWS "=" BWS ( token / quoted-string )
    let (i, key) = map(token, |s: &str| s.to_string()).parse(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = char('=')(i)?;
    let (i, _) = multispace0(i)?;
    let (i, value) = param_value(i)?;
    Ok((i, (key, value)))
}

fn token(i: &str) -> IResult<&str, &str> {
    // RFC 9110 §5.6.2: token = 1*tchar
    take_while1(|c: char| {
        matches!(
            c,
            '!' | '#'
                | '$'
                | '%'
                | '&'
                | '\''
                | '*'
                | '+'
                | '-'
                | '.'
                | '^'
                | '_'
                | '`'
                | '|'
                | '~'
        ) || c.is_ascii_alphanumeric()
    })(i)
}

fn param_value(i: &str) -> IResult<&str, String> {
    alt((quoted_value, map(token, |s: &str| s.to_string()))).parse(i)
}

/// Quoted string with backslash-escaped characters, per RFC 9110 `quoted-string`.
fn quoted_value(i: &str) -> IResult<&str, String> {
    delimited(
        char('"'),
        escaped_transform(none_of("\\\""), '\\', anychar),
        char('"'),
    )
    .parse(i)
}

pub fn build_authorization(
    credentials: &Credentials,
    challenge: &Challenge,
) -> Result<String, ProxyError> {
    match challenge.scheme.to_ascii_lowercase().as_str() {
        "basic" => Ok(basic_header(credentials)),
        other => Err(ProxyError::InvalidChallenge(format!(
            "unsupported auth scheme `{other}`"
        ))),
    }
}
