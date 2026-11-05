//! HTTP server adapter mapping WebDAV methods to server handler traits using the Actix Web framework.

use crate::webdav::error::DavError;
use crate::webdav::server::{DavHandler, Response};
use crate::webdav::{Acl, Depth, Propfind, Proppatch};
use actix_web::body::BoxBody;
use actix_web::error::ResponseError;
use actix_web::http::header;
use actix_web::web::Bytes;
use actix_web::{HttpRequest, HttpResponse, http};

/// A DAV error that can be returned directly from a handler. The framework
/// converts handler errors into this via `map_error`.
#[derive(Debug)]
pub struct DavResponder(pub (u16, DavError));

impl std::fmt::Display for DavResponder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.1)
    }
}

impl ResponseError for DavResponder {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::from_u16(self.0.0)
            .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.status_code())
            .content_type("application/xml; charset=utf-8")
            .body(dav_error_xml(&self.0.1))
    }
}

/// A minimal DAV error XML body generated from a [`DavError`].
fn dav_error_xml(err: &DavError) -> String {
    let body = crate::webdav::dav_error("server-error", err.to_string());
    String::from_utf8_lossy(&body.to_xml()).into_owned()
}

/// Construct a non-standard HTTP method (WebDAV) from a static name.
fn dav_method(name: &'static str) -> http::Method {
    http::Method::from_bytes(name.as_bytes()).unwrap_or(http::Method::GET)
}

/// Policy controlling which DAV methods the HTTP service exposes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DavServiceConfig {
    /// Whether `ACL` requests are forwarded to the underlying handler.
    pub dangerous_allow_acl_passthrough: bool,
    /// Whether only safe read methods are exposed.
    pub read_only: bool,
}

impl DavServiceConfig {
    /// ACL is exposed only when explicitly allowed and not overridden by
    /// read-only mode.
    pub fn acl_enabled(self) -> bool {
        self.dangerous_allow_acl_passthrough && !self.read_only
    }
}

/// Builder for [`DavServiceConfig`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DavServiceConfigBuilder {
    config: DavServiceConfig,
}

impl DavServiceConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn dangerous_allow_acl_passthrough(mut self, value: bool) -> Self {
        self.config.dangerous_allow_acl_passthrough = value;
        self
    }

    pub fn read_only(mut self, value: bool) -> Self {
        self.config.read_only = value;
        self
    }

    pub fn build(self) -> DavServiceConfig {
        self.config
    }
}

fn allow_value(config: DavServiceConfig) -> &'static str {
    if config.read_only {
        "OPTIONS, GET, HEAD, PROPFIND, REPORT"
    } else if config.acl_enabled() {
        "OPTIONS, GET, HEAD, PROPFIND, PROPPATCH, PUT, DELETE, MKCOL, ACL, REPORT"
    } else {
        "OPTIONS, GET, HEAD, PROPFIND, PROPPATCH, PUT, DELETE, MKCOL, REPORT"
    }
}

/// An actix-web service factory that wires DAV HTTP methods to a generic
/// [`DavHandler`]. Pass the returned scope to an actix `App::service`.
pub fn dav_service<H: DavHandler + 'static>(handler: H) -> actix_web::Scope {
    dav_service_with_config(handler, DavServiceConfig::default())
}

/// Like [`dav_service`], with explicit policy. Read-only mode routes only safe
/// read methods; every other method falls through to a `405` response and is
/// never forwarded to the handler.
pub fn dav_service_with_config<H: DavHandler + 'static>(
    handler: H,
    config: DavServiceConfig,
) -> actix_web::Scope {
    actix_web::web::scope("").configure(move |cfg| {
        cfg.app_data(actix_web::web::Data::new(handler))
            .app_data(actix_web::web::Data::new(config))
            .route("/{tail:.*}", actix_web::web::get().to(get::<H>))
            .route("/{tail:.*}", actix_web::web::head().to(head::<H>))
            .route(
                "/{tail:.*}",
                actix_web::web::route()
                    .method(dav_method("PROPFIND"))
                    .to(propfind::<H>),
            )
            .route(
                "/{tail:.*}",
                actix_web::web::route()
                    .method(dav_method("REPORT"))
                    .to(report::<H>),
            )
            .route(
                "/{tail:.*}",
                actix_web::web::route()
                    .method(http::Method::OPTIONS)
                    .to(options::<H>),
            );

        if !config.read_only {
            cfg.route("/{tail:.*}", actix_web::web::put().to(put::<H>))
                .route("/{tail:.*}", actix_web::web::delete().to(delete::<H>))
                .route(
                    "/{tail:.*}",
                    actix_web::web::route()
                        .method(dav_method("PROPPATCH"))
                        .to(proppatch::<H>),
                )
                .route(
                    "/{tail:.*}",
                    actix_web::web::route()
                        .method(dav_method("MKCOL"))
                        .to(mkcol::<H>),
                );
            if config.acl_enabled() {
                cfg.route(
                    "/{tail:.*}",
                    actix_web::web::route()
                        .method(dav_method("ACL"))
                        .to(acl::<H>),
                );
            }
        }

        cfg.route(
            "/{tail:.*}",
            actix_web::web::route().to(method_not_allowed::<H>),
        );
    })
}

fn path_of(req: &HttpRequest) -> String {
    req.path().to_string()
}

fn depth_of(req: &HttpRequest, default: Depth) -> Result<Depth, DavResponder> {
    match req.headers().get("depth") {
        Some(v) => match v.to_str().unwrap_or("") {
            "0" => Ok(Depth::Zero),
            "1" => Ok(Depth::One),
            "infinity" => Ok(Depth::Infinity),
            _ => Err(DavResponder((
                400,
                DavError::Invalid(format!(
                    "invalid Depth header: {}",
                    v.to_str().unwrap_or("")
                )),
            ))),
        },
        None => Ok(default),
    }
}

fn map_result<H: DavHandler>(
    handler: &H,
    result: Result<Response, H::Error>,
) -> Result<HttpResponse, DavResponder> {
    match result {
        Ok(resp) => Ok(serialize(resp)),
        Err(ref e) => Err(DavResponder(handler.map_error(e))),
    }
}

fn serialize(resp: Response) -> HttpResponse {
    match resp {
        Response::Multistatus(ms) => HttpResponse::MultiStatus()
            .content_type("application/xml; charset=utf-8")
            .body(ms.to_xml()),
        Response::Resource(res) => {
            let mut builder = HttpResponse::Ok();
            builder.content_type(res.content_type.as_str());
            if let Some(etag) = res.etag
                && let Ok(val) = actix_web::http::header::HeaderValue::from_str(&etag)
            {
                builder.insert_header((header::ETAG, val));
            }
            builder.body(res.body)
        }
        Response::Status(code) => HttpResponse::build(
            actix_web::http::StatusCode::from_u16(code)
                .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR),
        )
        .finish(),
        Response::StatusBody {
            status,
            content_type,
            body,
        } => HttpResponse::build(
            actix_web::http::StatusCode::from_u16(status)
                .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR),
        )
        .content_type(content_type.as_str())
        .body(body),
    }
}

async fn get<H: DavHandler>(
    req: HttpRequest,
    state: actix_web::web::Data<H>,
) -> Result<HttpResponse, DavResponder> {
    let handler = &**state;
    let path = path_of(&req);
    map_result(handler, handler.get(&path).await)
}

async fn head<H: DavHandler>(
    req: HttpRequest,
    state: actix_web::web::Data<H>,
) -> Result<HttpResponse, DavResponder> {
    let handler = &**state;
    let path = path_of(&req);
    map_result(handler, handler.get(&path).await)
}

async fn put<H: DavHandler>(
    req: HttpRequest,
    body: Bytes,
    state: actix_web::web::Data<H>,
) -> Result<HttpResponse, DavResponder> {
    let handler = &**state;
    let path = path_of(&req);
    let content_type = req
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();
    let etag = req
        .headers()
        .get(header::IF_MATCH)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let resource = crate::webdav::resource::Resource {
        content_type,
        etag,
        body: body.to_vec(),
    };
    map_result(handler, handler.put(&path, resource).await)
}

async fn delete<H: DavHandler>(
    req: HttpRequest,
    state: actix_web::web::Data<H>,
) -> Result<HttpResponse, DavResponder> {
    let handler = &**state;
    let path = path_of(&req);
    map_result(handler, handler.delete(&path).await)
}

async fn propfind<H: DavHandler>(
    req: HttpRequest,
    body: Bytes,
    state: actix_web::web::Data<H>,
) -> Result<HttpResponse, DavResponder> {
    let depth = depth_of(&req, Depth::Infinity)?;
    let propfind = match Propfind::from_xml(&body) {
        Ok(p) => p,
        Err(e) => return Err(DavResponder((400, e))),
    };
    let handler = &**state;
    let path = path_of(&req);
    map_result(handler, handler.propfind(&path, depth, propfind).await)
}

async fn proppatch<H: DavHandler>(
    req: HttpRequest,
    body: Bytes,
    state: actix_web::web::Data<H>,
) -> Result<HttpResponse, DavResponder> {
    let path = path_of(&req);
    let proppatch = match Proppatch::from_xml(&body) {
        Ok(p) => p,
        Err(_) => {
            return Err(DavResponder((
                400,
                DavError::XmlParse("bad proppatch".into()),
            )));
        }
    };
    let handler = &**state;
    map_result(handler, handler.proppatch(&path, proppatch).await)
}

async fn mkcol<H: DavHandler>(
    req: HttpRequest,
    body: Bytes,
    state: actix_web::web::Data<H>,
) -> Result<HttpResponse, DavResponder> {
    let _ = body;
    let handler = &**state;
    let path = path_of(&req);
    map_result(handler, handler.mkcol(&path, None).await)
}

async fn acl<H: DavHandler>(
    req: HttpRequest,
    body: Bytes,
    state: actix_web::web::Data<H>,
) -> Result<HttpResponse, DavResponder> {
    let acl_body = match Acl::from_xml(&body) {
        Ok(a) => a,
        Err(e) => return Err(DavResponder((400, e))),
    };
    let handler = &**state;
    let path = path_of(&req);
    map_result(handler, handler.acl(&path, &acl_body).await)
}

async fn report<H: DavHandler>(
    req: HttpRequest,
    body: Bytes,
    state: actix_web::web::Data<H>,
) -> Result<HttpResponse, DavResponder> {
    let depth = depth_of(&req, Depth::Zero)?;
    let handler = &**state;
    let path = path_of(&req);
    map_result(handler, handler.report(&path, depth, &body).await)
}

async fn options<H: DavHandler>(
    req: HttpRequest,
    state: actix_web::web::Data<H>,
    config: actix_web::web::Data<DavServiceConfig>,
) -> HttpResponse {
    let _ = req;
    let caps = state.capabilities();
    HttpResponse::Ok()
        .insert_header(("DAV", caps.header_value()))
        .insert_header(("Allow", allow_value(**config)))
        .finish()
}

async fn method_not_allowed<H: DavHandler>(
    _state: actix_web::web::Data<H>,
    config: actix_web::web::Data<DavServiceConfig>,
) -> HttpResponse {
    HttpResponse::MethodNotAllowed()
        .insert_header(("Allow", allow_value(**config)))
        .finish()
}
