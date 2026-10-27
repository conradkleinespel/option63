//! WebDAV HTTP client implementation supporting standard and extended DAV methods with authentication.

use crate::webdav::auth::basic_header;
use crate::webdav::error::DavError;
use crate::webdav::resource::Resource;
use crate::webdav::{Acl, Depth, Multistatus, Propfind, Proppatch, SyncCollection};
use reqwest::{Client, Method, StatusCode, Url};

/// A basic-auth credential pair.
#[derive(Clone, Debug)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

/// A WebDAV client built on `reqwest`. It speaks the shared protocol types
/// directly, so a server handler can forward to another DAV server by simply
/// calling these methods (the proxy use case).
#[derive(Clone)]
pub struct DavClient {
    http: Client,
    /// The base URL (origin + optional path prefix) for the server.
    base: Url,
    credentials: Option<Credentials>,
}

impl DavClient {
    pub fn new(base: Url) -> Self {
        Self {
            http: Client::new(),
            base,
            credentials: None,
        }
    }

    pub fn with_credentials(mut self, c: Credentials) -> Self {
        self.credentials = Some(c);
        self
    }

    /// Resolve a server path against the base URL, preserving any path prefix
    /// present in `base`.
    fn url(&self, path: &str) -> Url {
        let mut base = self.base.clone();
        let base_path = base.path().trim_end_matches('/');
        let combined = match base_path {
            "" | "/" => path.trim_start_matches('/').to_string(),
            prefix => format!("{prefix}/{}", path.trim_start_matches('/')),
        };
        base.set_path(&combined);
        base
    }

    fn apply_auth(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match &self.credentials {
            Some(creds) => builder.header(
                "authorization",
                basic_header(&creds.username, &creds.password),
            ),
            None => builder,
        }
    }

    pub async fn propfind(
        &self,
        path: &str,
        depth: Depth,
        body: &Propfind,
    ) -> Result<Multistatus, DavError> {
        let url = self.url(path);
        let builder = self
            .http
            .request(dav_method("PROPFIND")?, url)
            .header("Depth", depth_value(depth))
            .header("Content-Type", "application/xml; charset=utf-8")
            .body(body.to_xml());
        let resp = self
            .apply_auth(builder)
            .send()
            .await
            .map_err(DavError::Http)?;
        let status = resp.status();
        let bytes = resp.bytes().await.map_err(DavError::Http)?;
        if status != StatusCode::MULTI_STATUS {
            return Err(self.error_for_status(status, &bytes));
        }
        Multistatus::from_xml(&bytes)
    }

    pub async fn get(&self, path: &str) -> Result<Resource, DavError> {
        let url = self.url(path);
        let builder = self.http.request(Method::GET, url);
        let resp = self
            .apply_auth(builder)
            .send()
            .await
            .map_err(DavError::Http)?;
        let status = resp.status();
        let headers = resp.headers().clone();
        let bytes = resp.bytes().await.map_err(DavError::Http)?;
        if !status.is_success() {
            return Err(self.error_for_status(status, &bytes));
        }
        let content_type = headers
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();
        let etag = headers
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        Ok(Resource {
            content_type,
            etag,
            body: bytes.to_vec(),
        })
    }

    pub async fn put(&self, path: &str, resource: &Resource) -> Result<u16, DavError> {
        let url = self.url(path);
        let builder = self
            .http
            .request(Method::PUT, url)
            .header("Content-Type", resource.content_type.as_str())
            .body(resource.body.clone());
        let resp = self
            .apply_auth(builder)
            .send()
            .await
            .map_err(DavError::Http)?;
        let status = resp.status();
        let bytes = resp.bytes().await.map_err(DavError::Http)?;
        if !status.is_success() {
            return Err(self.error_for_status(status, &bytes));
        }
        Ok(status.as_u16())
    }

    pub async fn delete(&self, path: &str) -> Result<u16, DavError> {
        let url = self.url(path);
        let builder = self.http.request(Method::DELETE, url);
        let resp = self
            .apply_auth(builder)
            .send()
            .await
            .map_err(DavError::Http)?;
        let status = resp.status();
        let bytes = resp.bytes().await.map_err(DavError::Http)?;
        if !status.is_success() {
            return Err(self.error_for_status(status, &bytes));
        }
        Ok(status.as_u16())
    }

    pub async fn mkcol(&self, path: &str) -> Result<u16, DavError> {
        let url = self.url(path);
        let builder = self.http.request(dav_method("MKCOL")?, url);
        let resp = self
            .apply_auth(builder)
            .send()
            .await
            .map_err(DavError::Http)?;
        let status = resp.status();
        let bytes = resp.bytes().await.map_err(DavError::Http)?;
        if !status.is_success() {
            return Err(self.error_for_status(status, &bytes));
        }
        Ok(status.as_u16())
    }

    pub async fn proppatch(&self, path: &str, body: &Proppatch) -> Result<Multistatus, DavError> {
        let url = self.url(path);
        let xml = body.to_xml()?;
        let builder = self
            .http
            .request(dav_method("PROPPATCH")?, url)
            .header("Content-Type", "application/xml; charset=utf-8")
            .body(xml);
        let resp = self
            .apply_auth(builder)
            .send()
            .await
            .map_err(DavError::Http)?;
        let status = resp.status();
        let bytes = resp.bytes().await.map_err(DavError::Http)?;
        if status != StatusCode::MULTI_STATUS {
            return Err(self.error_for_status(status, &bytes));
        }
        Multistatus::from_xml(&bytes)
    }

    /// Send a generic REPORT request (RFC 3253 §3.6). The `body` is the
    /// raw XML report body. Returns the parsed `DAV:multistatus` response.
    pub async fn report(
        &self,
        path: &str,
        depth: Depth,
        body: &[u8],
    ) -> Result<Multistatus, DavError> {
        let url = self.url(path);
        let builder = self
            .http
            .request(dav_method("REPORT")?, url)
            .header("Depth", depth_value(depth))
            .header("Content-Type", "application/xml; charset=utf-8")
            .body(body.to_vec());
        let resp = self
            .apply_auth(builder)
            .send()
            .await
            .map_err(DavError::Http)?;
        let status = resp.status();
        let bytes = resp.bytes().await.map_err(DavError::Http)?;
        if status != StatusCode::MULTI_STATUS {
            return Err(self.error_for_status(status, &bytes));
        }
        Multistatus::from_xml(&bytes)
    }

    /// Send a `DAV:sync-collection` report (RFC 6578). Always uses
    /// `Depth: 0` per RFC 6578 §3.2.
    pub async fn sync_collection(
        &self,
        path: &str,
        request: &SyncCollection,
    ) -> Result<Multistatus, DavError> {
        self.report(path, Depth::Zero, &request.to_xml()).await
    }

    /// Modify the access control list of a resource (RFC 3744 §8.1).
    /// On success the server returns 200 OK with no body.
    pub async fn acl(&self, path: &str, acl: &Acl) -> Result<u16, DavError> {
        let url = self.url(path);
        let builder = self
            .http
            .request(dav_method("ACL")?, url)
            .header("Content-Type", "application/xml; charset=utf-8")
            .body(acl.to_xml());
        let resp = self
            .apply_auth(builder)
            .send()
            .await
            .map_err(DavError::Http)?;
        let status = resp.status();
        let bytes = resp.bytes().await.map_err(DavError::Http)?;
        if !status.is_success() {
            return Err(self.error_for_status(status, &bytes));
        }
        Ok(status.as_u16())
    }

    /// Send an OPTIONS request and return the `DAV` compliance header value.
    pub async fn options(&self, path: &str) -> Result<String, DavError> {
        let url = self.url(path);
        let builder = self.http.request(Method::OPTIONS, url);
        let resp = self
            .apply_auth(builder)
            .send()
            .await
            .map_err(DavError::Http)?;
        let status = resp.status();
        let headers = resp.headers().clone();
        let bytes = resp.bytes().await.map_err(DavError::Http)?;
        if !status.is_success() {
            return Err(self.error_for_status(status, &bytes));
        }
        Ok(headers
            .get("dav")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string())
    }

    fn error_for_status(&self, status: StatusCode, body: &[u8]) -> DavError {
        let _ = body;
        DavError::HttpStatus(status.as_u16())
    }
}

fn depth_value(depth: Depth) -> &'static str {
    match depth {
        Depth::Zero => "0",
        Depth::One => "1",
        Depth::Infinity => "infinity",
    }
}

fn dav_method(name: &str) -> Result<Method, DavError> {
    Method::from_bytes(name.as_bytes()).map_err(|e| DavError::Invalid(e.to_string()))
}
