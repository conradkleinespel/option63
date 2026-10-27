//! WebDAV server handler traits, method dispatch interfaces, and capability advertisements.

use crate::webdav::error::DavError;
use crate::webdav::resource::Resource;
use crate::webdav::{Acl, Depth, Multistatus, Propfind, Proppatch};

/// A set of DAV compliance classes and extension namespaces advertised in the
/// `DAV:` response header to OPTIONS.
#[derive(Clone, Debug, Default)]
pub struct DavCapabilities {
    /// Compliance classes, e.g. `["1", "3"]`.
    pub classes: Vec<String>,
    /// Extension namespaces, e.g. CardDAV/CalDAV namespace URIs.
    pub namespace: Vec<String>,
}

impl DavCapabilities {
    /// The value of the `DAV:` header, e.g. `1, 3, addressbook`.
    pub fn header_value(&self) -> String {
        let mut parts = self.classes.clone();
        parts.extend(self.namespace.clone());
        parts.join(", ")
    }
}

/// The outcome of a DAV method that the server framework serializes into an
/// HTTP response.
#[derive(Debug)]
pub enum Response {
    /// A 207 Multi-Status body.
    Multistatus(Multistatus),
    /// A resource (GET).
    Resource(Resource),
    /// A plain status with no body (e.g. 204 No Content).
    Status(u16),
    /// A plain status with a body and content type.
    StatusBody {
        status: u16,
        content_type: String,
        body: Vec<u8>,
    },
}

/// The server-side DAV handler trait. Each method corresponds to a DAV
/// operation over the shared protocol types, so a client-backed proxy handler
/// is a trivial adapter that forwards each call to the client.
#[allow(async_fn_in_trait)]
pub trait DavHandler: Send + Sync + 'static {
    type Error: Into<DavError> + Send + Sync + 'static;

    /// Map an implementor error to (HTTP status, DAV error body).
    fn map_error(&self, err: &Self::Error) -> (u16, DavError);

    /// The compliance classes / namespaces to advertise via OPTIONS.
    fn capabilities(&self) -> DavCapabilities {
        DavCapabilities {
            classes: vec!["1".into(), "3".into()],
            namespace: Vec::new(),
        }
    }

    /// Whether the server accepts `ACL` (RFC 3744). When false, ACL requests
    /// are refused and `ACL` is not advertised in the OPTIONS `Allow` header.
    fn acl_enabled(&self) -> bool {
        true
    }

    async fn propfind(
        &self,
        path: &str,
        depth: Depth,
        request: Propfind,
    ) -> Result<Response, Self::Error>;

    async fn proppatch(&self, path: &str, request: Proppatch) -> Result<Response, Self::Error>;

    async fn get(&self, path: &str) -> Result<Response, Self::Error>;

    async fn put(&self, path: &str, resource: Resource) -> Result<Response, Self::Error>;

    async fn delete(&self, path: &str) -> Result<Response, Self::Error>;

    async fn mkcol(&self, path: &str, request: Option<Proppatch>) -> Result<Response, Self::Error>;

    async fn acl(&self, path: &str, acl: &Acl) -> Result<Response, Self::Error>;

    async fn report(&self, path: &str, depth: Depth, body: &[u8]) -> Result<Response, Self::Error>;
}
