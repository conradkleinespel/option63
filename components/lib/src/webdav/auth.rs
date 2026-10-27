//! HTTP Basic authentication header parsing and server authentication middleware.

use crate::webdav::client::Credentials;
use crate::webdav::error::DavError;
use actix_web::body::MessageBody;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::header;
use actix_web::{Error, HttpResponse};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use nom::IResult;
use nom::Parser;
use nom::branch::alt;
use nom::bytes::complete::{escaped_transform, take_while1};
use nom::character::complete::{anychar, char, multispace0, none_of};
use nom::combinator::map;
use nom::multi::separated_list0;
use nom::sequence::delimited;
use std::future::{Future, Ready, ready};
use std::pin::Pin;
use std::task::{Context, Poll};

/// A parsed authentication challenge from a `WWW-Authenticate` header.
#[derive(Debug, Clone)]
pub struct Challenge {
    pub scheme: String,
}

/// Build a `Basic` authorization header value from credentials.
pub fn basic_header(username: &str, password: &str) -> String {
    let combined = format!("{username}:{password}");
    format!("Basic {}", BASE64.encode(combined.as_bytes()))
}

/// Parse a `WWW-Authenticate` header and select the `basic` challenge.
pub fn parse_www_authenticate(header: &str) -> Result<Challenge, DavError> {
    let challenges = parse_challenges(header)?;

    let challenge = challenges
        .iter()
        .find(|c| c.scheme.eq_ignore_ascii_case("basic"))
        .ok_or_else(|| DavError::InvalidChallenge(format!("no `basic` challenge in `{header}`")))?;

    Ok(Challenge {
        scheme: challenge.scheme.clone(),
    })
}

/// Parse a `WWW-Authenticate` header into all of its challenges, regardless of
/// scheme. Per RFC 9110 (§11.2) a field is `[ challenge *( OWS "," OWS
/// challenge ) ]`.
fn parse_challenges(header: &str) -> Result<Vec<ParsedChallenge>, DavError> {
    let (rest, challenges) = challenges(header)
        .map_err(|e| DavError::InvalidChallenge(format!("failed to parse `{header}`: {e}")))?;

    if !rest.trim().is_empty() {
        return Err(DavError::InvalidChallenge(format!(
            "trailing data after challenge in `{header}`: `{rest}`"
        )));
    }

    Ok(challenges)
}

/// Per RFC 9110 (§11.2), a `WWW-Authenticate` field is `[ challenge *( OWS "," OWS
/// challenge ) ]`, where each challenge is
/// `auth-scheme [ 1*SP ( token68 / [ auth-param *( OWS "," OWS auth-param ) ] ) ]`.
#[derive(Debug, Clone)]
struct ParsedChallenge {
    scheme: String,
    /// Parsed `auth-param`s (realm, nonce, ...). `basic` (the only supported
    /// scheme) does not use them, but they are parsed to validate structure.
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

fn quoted_value(i: &str) -> IResult<&str, String> {
    delimited(
        char('"'),
        escaped_transform(none_of("\\\""), '\\', anychar),
        char('"'),
    )
    .parse(i)
}

/// Build an authorization header from credentials and a challenge. Only
/// `Basic` is supported.
pub fn build_authorization(
    username: &str,
    password: &str,
    challenge: &Challenge,
) -> Result<String, DavError> {
    match challenge.scheme.to_ascii_lowercase().as_str() {
        "basic" => Ok(basic_header(username, password)),
        other => Err(DavError::InvalidChallenge(format!(
            "unsupported auth scheme `{other}`"
        ))),
    }
}

/// Parse an `Authorization` request header value of the form `Basic <base64>`.
/// Returns the decoded `(username, password)`, or `None` if the value is not a
/// Basic scheme, not valid base64, or has no colon-separated password.
pub fn parse_basic_authorization(value: &str) -> Option<(String, String)> {
    let (scheme, base64_encoded) = value.split_once(' ')?;
    if !scheme.eq_ignore_ascii_case("basic") {
        return None;
    }
    let decoded = BASE64.decode(base64_encoded.trim().as_bytes()).ok()?;
    let combined = String::from_utf8(decoded).ok()?;
    let (username, password) = combined.split_once(':')?;
    if username.is_empty() {
        return None;
    }
    Some((username.to_string(), password.to_string()))
}

/// Constant-time byte comparison. Only the length difference is observable.
/// TODO: replace with crate constant_time_eq = "0.6"
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Whether the presented username/password match the expected credentials.
pub fn credentials_match(expected: &Credentials, username: &str, password: &str) -> bool {
    constant_time_eq(expected.username.as_bytes(), username.as_bytes())
        && constant_time_eq(expected.password.as_bytes(), password.as_bytes())
}

/// An actix-web error mapping to `401 Unauthorized` with a Basic challenge.
/// Used by [`BasicAuth`] so unauthenticated requests are refused before the
/// wrapped service runs.
#[derive(Debug)]
pub struct Unauthorized(pub &'static str);

impl std::fmt::Display for Unauthorized {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "authentication required")
    }
}

impl actix_web::ResponseError for Unauthorized {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::UNAUTHORIZED
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::Unauthorized()
            .insert_header(("WWW-Authenticate", format!("Basic realm=\"{}\"", self.0)))
            .finish()
    }
}

/// An actix-web middleware that requires valid Basic credentials on every
/// request before the wrapped service runs. Unauthenticated requests receive
/// `401` with a `WWW-Authenticate` challenge and are never forwarded.
#[derive(Clone)]
pub struct BasicAuth {
    expected: Credentials,
    realm: &'static str,
}

impl BasicAuth {
    pub fn new(expected: Credentials) -> Self {
        Self {
            expected,
            realm: "carddav-proxy",
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for BasicAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = BasicAuthService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(BasicAuthService {
            service,
            expected: self.expected.clone(),
            realm: self.realm,
        }))
    }
}

pub struct BasicAuthService<S> {
    service: S,
    expected: Credentials,
    realm: &'static str,
}

impl<S, B> Service<ServiceRequest> for BasicAuthService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let authorized = match req.headers().get(header::AUTHORIZATION) {
            Some(value) => value
                .to_str()
                .ok()
                .and_then(parse_basic_authorization)
                .map(|(username, password)| credentials_match(&self.expected, &username, &password))
                .unwrap_or(false),
            None => false,
        };
        if !authorized {
            let realm = self.realm;
            return Box::pin(async move { Err(Error::from(Unauthorized(realm))) });
        }
        let fut = self.service.call(req);
        Box::pin(fut)
    }
}
