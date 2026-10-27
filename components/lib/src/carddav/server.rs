//! Server traits, request handling interfaces, and capability advertisements for CardDAV services.

use crate::carddav::protocol::{AddressbookMultiget, AddressbookQuery};
use crate::webdav::Depth;
use crate::webdav::server::{DavCapabilities, DavHandler, Response};

/// A CardDAV server handler. Extends [`DavHandler`] with CardDAV-specific
/// report methods (`addressbook-query`, `addressbook-multiget`).
#[allow(async_fn_in_trait)]
pub trait CarddavHandler: DavHandler {
    /// Handle a `C:addressbook-query` report (RFC 6352 §8.6).
    ///
    /// The scope of the query is determined by the `depth` header.
    /// The response must be a `DAV:multistatus` with one `D:response` per
    /// matching address object resource.
    async fn addressbook_query(
        &self,
        path: &str,
        depth: Depth,
        query: AddressbookQuery,
    ) -> Result<Response, Self::Error>;

    /// Handle a `C:addressbook-multiget` report (RFC 6352 §8.7).
    ///
    /// The scope is the set of `D:href` elements in the request body.
    /// The response must contain a `D:response` for each requested href.
    async fn addressbook_multiget(
        &self,
        path: &str,
        multiget: AddressbookMultiget,
    ) -> Result<Response, Self::Error>;
}

/// CardDAV server capabilities for the `DAV:` header.
///
/// CardDAV servers advertise class `1`, `3`, and the `addressbook` extension
/// namespace (RFC 6352 §5.1).
pub fn carddav_capabilities() -> DavCapabilities {
    DavCapabilities {
        classes: vec!["1".into(), "3".into()],
        namespace: vec!["addressbook".into()],
    }
}
