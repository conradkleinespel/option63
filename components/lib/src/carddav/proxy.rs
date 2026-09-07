use crate::carddav::auth::{Challenge, build_authorization, parse_www_authenticate};
use crate::carddav::creds::Credentials;
use crate::carddav::error::ProxyError;
use actix_web::web;
use actix_web::{HttpRequest, HttpResponse};
use reqwest::{Client, RequestBuilder, Url};
use uuid::Uuid;

pub struct AppState {
    pub upstream: Url,
    pub client: Client,
    pub credentials: Credentials,
    pub dangerous_log_bodies: bool,
}

impl AppState {
    pub fn new(
        upstream: Url,
        client: Client,
        credentials: Credentials,
        dangerous_log_bodies: bool,
    ) -> Self {
        Self {
            upstream,
            client,
            credentials,
            dangerous_log_bodies,
        }
    }
}

fn is_hop_by_hop(name: &str) -> bool {
    matches!(
        name,
        "connection"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailer"
            | "transfer-encoding"
            | "upgrade"
    )
}

/// Headers that the underlying `reqwest` client recomputes from the request
/// body/target, so forwarding the caller-supplied value would conflict.
fn is_recomputed_by_underlying_client(name: &str) -> bool {
    matches!(name, "content-length" | "host")
}

/// Decide whether a request header should be forwarded to the upstream.
fn should_forward_request_header(name: &str) -> bool {
    !is_hop_by_hop(name) && !is_recomputed_by_underlying_client(name)
}

fn build_target(upstream: &Url, path: &str, query: Option<&str>) -> Url {
    let mut target = upstream.clone();
    target.set_path(path);
    target.set_query(query);
    target
}

fn build_request(
    client: &Client,
    method: &reqwest::Method,
    target: &Url,
    headers: &[(String, Vec<u8>)],
    body: &web::Bytes,
    auth: Option<&str>,
) -> RequestBuilder {
    let mut builder = client.request(method.clone(), target.clone());
    for (name, value) in headers {
        builder = builder.header(name.as_str(), value.as_slice());
    }
    if let Some(auth) = auth {
        builder = builder.header("authorization", auth);
    }
    if !body.is_empty() {
        builder = builder.body(body.to_vec());
    }
    builder
}

async fn to_http_response(resp: reqwest::Response, dangerous_log_bodies: bool) -> HttpResponse {
    let status = actix_web::http::StatusCode::from_u16(resp.status().as_u16())
        .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
    let mut builder = HttpResponse::build(status);
    for (name, value) in resp.headers() {
        if !is_hop_by_hop(name.as_str()) {
            builder.insert_header((name.as_str(), value.as_bytes().to_vec()));
        }
    }
    let resp_body = resp.bytes().await.unwrap_or_default();
    if dangerous_log_bodies {
        tracing::debug!(
            response_body = %String::from_utf8_lossy(&resp_body),
            "response body",
        );
    }
    builder.body(resp_body.to_vec())
}

async fn forward(
    req: HttpRequest,
    body: web::Bytes,
    state: web::Data<AppState>,
    method_str: &str,
) -> Result<HttpResponse, ProxyError> {
    let target = build_target(&state.upstream, req.uri().path(), req.uri().query());
    let method = reqwest::Method::from_bytes(method_str.as_bytes())
        .map_err(|e| ProxyError::InvalidMethod(method_str.to_string(), e.to_string()))?;

    let headers: Vec<(String, Vec<u8>)> = req
        .headers()
        .iter()
        .filter(|(name, _)| should_forward_request_header(name.as_str()))
        .map(|(name, value)| (name.as_str().to_string(), value.as_bytes().to_vec()))
        .collect();

    if state.dangerous_log_bodies {
        tracing::debug!(
            request_body = %String::from_utf8_lossy(&body),
            "request body",
        );
    }

    let resp = build_request(&state.client, &method, &target, &headers, &body, None)
        .send()
        .await
        .map_err(ProxyError::Upstream)?;

    tracing::debug!(
        target = %target,
        method = %method_str,
        first_status = resp.status().as_u16(),
        "request forwarded to upstream"
    );

    if resp.status().as_u16() == 401 {
        let basic_challenge = select_basic_challenge(resp.headers().get_all("www-authenticate"));

        if let Some(challenge) = basic_challenge
            && let Ok(auth) = build_authorization(&state.credentials, &challenge)
        {
            tracing::info!(
                challenge_scheme = %challenge.scheme,
                "upstream requested authentication; retrying with credentials"
            );
            let retry = build_request(
                &state.client,
                &method,
                &target,
                &headers,
                &body,
                Some(&auth),
            )
            .send()
            .await
            .map_err(ProxyError::Upstream)?;
            if retry.status().as_u16() == 401 {
                tracing::info!(
                    status = retry.status().as_u16(),
                    "upstream rejected the provided credentials; forwarding so the client can re-authenticate"
                );
            }
            return Ok(to_http_response(retry, state.dangerous_log_bodies).await);
        }

        tracing::info!("upstream requested authentication but offered no basic challenge");
        return auth_error_response(
            "upstream requested an authentication scheme the proxy does not support",
        );
    }

    Ok(to_http_response(resp, state.dangerous_log_bodies).await)
}

fn auth_error_response(message: &str) -> Result<HttpResponse, ProxyError> {
    let body = error_dav_xml("auth-challenge-unsupported", message)?;
    Ok(HttpResponse::BadGateway()
        .content_type("application/xml; charset=utf-8")
        .body(body))
}

fn error_dav_xml(condition: &str, message: &str) -> Result<String, ProxyError> {
    use quick_xml::Writer;
    use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};

    const DAV_NS: &str = "DAV:";
    const PROXY_NS: &str = "https://option63.eu/ns/carddav-proxy";

    let mut writer = Writer::new(Vec::new());

    writer
        .write_event(Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))
        .map_err(ProxyError::XmlSerialize)?;

    let mut root = BytesStart::new("D:error");
    root.push_attribute(("xmlns:D", DAV_NS));
    root.push_attribute(("xmlns:o63", PROXY_NS));
    writer
        .write_event(Event::Start(root))
        .map_err(ProxyError::XmlSerialize)?;

    writer
        .write_event(Event::Empty(BytesStart::new(format!("o63:{condition}"))))
        .map_err(ProxyError::XmlSerialize)?;

    writer
        .write_event(Event::Start(BytesStart::new("D:responsedescription")))
        .map_err(ProxyError::XmlSerialize)?;
    writer
        .write_event(Event::Text(BytesText::new(message)))
        .map_err(ProxyError::XmlSerialize)?;
    writer
        .write_event(Event::End(BytesEnd::new("D:responsedescription")))
        .map_err(ProxyError::XmlSerialize)?;

    writer
        .write_event(Event::End(BytesEnd::new("D:error")))
        .map_err(ProxyError::XmlSerialize)?;

    String::from_utf8(writer.into_inner())
        .map_err(|_| ProxyError::XmlSerialize(std::io::Error::other("invalid UTF-8")))
}

fn select_basic_challenge<'a, I>(challenges: I) -> Option<Challenge>
where
    I: IntoIterator<Item = &'a reqwest::header::HeaderValue>,
{
    challenges
        .into_iter()
        .filter_map(|value| value.to_str().ok())
        .filter_map(|challenge_str| parse_www_authenticate(challenge_str).ok())
        .find(|challenge| challenge.scheme.eq_ignore_ascii_case("basic"))
}

async fn handle(
    req: HttpRequest,
    body: web::Bytes,
    state: web::Data<AppState>,
    method: &str,
) -> HttpResponse {
    let request_id = Uuid::new_v4();
    let span = tracing::info_span!(
        "carddav_request",
        request_id = %request_id,
        method = %method,
        path = %req.path(),
    );
    let _guard = span.enter();

    match forward(req, body, state, method).await {
        Ok(resp) => {
            tracing::info!(status = resp.status().as_u16(), "response returned");
            resp
        }
        Err(err) => {
            tracing::error!(error = %err, "upstream request failed");
            let message = format!("upstream: {err}");
            let body = error_dav_xml("upstream-unreachable", &message).unwrap_or_default();
            HttpResponse::BadGateway()
                .content_type("application/xml; charset=utf-8")
                .body(body)
        }
    }
}

macro_rules! dav_handler {
    ($name:ident, $method:literal) => {
        pub async fn $name(
            req: HttpRequest,
            body: web::Bytes,
            state: web::Data<AppState>,
        ) -> HttpResponse {
            handle(req, body, state, $method).await
        }
    };
}

dav_handler!(propfind, "PROPFIND");
dav_handler!(proppatch, "PROPPATCH");
dav_handler!(mkcol, "MKCOL");
dav_handler!(get, "GET");
dav_handler!(head, "HEAD");
dav_handler!(post, "POST");
dav_handler!(put, "PUT");
dav_handler!(delete, "DELETE");
dav_handler!(copy, "COPY");
dav_handler!(move_method, "MOVE");
dav_handler!(lock, "LOCK");
dav_handler!(unlock, "UNLOCK");
dav_handler!(report, "REPORT");
dav_handler!(options, "OPTIONS");

pub async fn catch_all(
    req: HttpRequest,
    body: web::Bytes,
    state: web::Data<AppState>,
) -> HttpResponse {
    let method = req.method().as_str().to_string();
    handle(req, body, state, &method).await
}
