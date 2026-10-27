//! Reverse proxy server for CardDAV.

use crate::carddav::client::CarddavClient;
use crate::carddav::protocol::{AddressbookMultiget, AddressbookQuery, parse_report};
use crate::carddav::server::{CarddavHandler, carddav_capabilities};
use crate::webdav::error::DavError;
use crate::webdav::resource::Resource;
use crate::webdav::server::{DavCapabilities, DavHandler, Response};
use crate::webdav::{Acl, Depth, Propfind, Proppatch, SyncCollection};

/// A client-backed CardDAV proxy handler. Implements [`DavHandler`] and
/// [`CarddavHandler`] by delegating every method to a [`CarddavClient`],
/// which speaks to an upstream server.
#[derive(Clone)]
pub struct ProxyHandler {
    client: CarddavClient,
    dangerous_allow_acl_passthrough: bool,
}

impl ProxyHandler {
    pub fn new(client: CarddavClient) -> Self {
        Self {
            client,
            dangerous_allow_acl_passthrough: false,
        }
    }

    pub fn new_with_acl(client: CarddavClient, dangerous_allow_acl_passthrough: bool) -> Self {
        Self {
            client,
            dangerous_allow_acl_passthrough,
        }
    }

    /// Access the underlying client (for testing or inspection).
    pub fn client(&self) -> &CarddavClient {
        &self.client
    }
}

impl DavHandler for ProxyHandler {
    type Error = DavError;

    fn map_error(&self, err: &Self::Error) -> (u16, DavError) {
        match err {
            DavError::HttpStatus(404) => (404, DavError::HttpStatus(404)),
            DavError::HttpStatus(code) => (*code, DavError::HttpStatus(*code)),
            other => (502, DavError::Invalid(other.to_string())),
        }
    }

    fn capabilities(&self) -> DavCapabilities {
        carddav_capabilities()
    }

    fn acl_enabled(&self) -> bool {
        self.dangerous_allow_acl_passthrough
    }

    async fn propfind(
        &self,
        path: &str,
        depth: Depth,
        request: Propfind,
    ) -> Result<Response, Self::Error> {
        let ms = self.client.dav().propfind(path, depth, &request).await?;
        Ok(Response::Multistatus(ms))
    }

    async fn proppatch(&self, path: &str, request: Proppatch) -> Result<Response, Self::Error> {
        let ms = self.client.dav().proppatch(path, &request).await?;
        Ok(Response::Multistatus(ms))
    }

    async fn get(&self, path: &str) -> Result<Response, Self::Error> {
        let resource = self.client.dav().get(path).await?;
        Ok(Response::Resource(resource))
    }

    async fn put(&self, path: &str, resource: Resource) -> Result<Response, Self::Error> {
        let status = self.client.dav().put(path, &resource).await?;
        Ok(Response::Status(status))
    }

    async fn delete(&self, path: &str) -> Result<Response, Self::Error> {
        let status = self.client.dav().delete(path).await?;
        Ok(Response::Status(status))
    }

    async fn mkcol(
        &self,
        path: &str,
        _request: Option<Proppatch>,
    ) -> Result<Response, Self::Error> {
        let status = self.client.dav().mkcol(path).await?;
        Ok(Response::Status(status))
    }

    async fn acl(&self, path: &str, acl: &Acl) -> Result<Response, Self::Error> {
        if !self.dangerous_allow_acl_passthrough {
            return Ok(Response::Status(405));
        }
        let status = self.client.dav().acl(path, acl).await?;
        Ok(Response::Status(status))
    }

    async fn report(&self, path: &str, depth: Depth, body: &[u8]) -> Result<Response, Self::Error> {
        if let Ok(sc) = SyncCollection::from_xml(body) {
            let ms = self.client.dav().sync_collection(path, &sc).await?;
            return Ok(Response::Multistatus(ms));
        }
        let report = parse_report(body)?;
        match report {
            crate::carddav::protocol::CarddavReport::AddressbookQuery(query) => {
                self.addressbook_query(path, depth, query).await
            }
            crate::carddav::protocol::CarddavReport::AddressbookMultiget(multiget) => {
                self.addressbook_multiget(path, multiget).await
            }
        }
    }
}

impl CarddavHandler for ProxyHandler {
    async fn addressbook_query(
        &self,
        path: &str,
        depth: Depth,
        query: AddressbookQuery,
    ) -> Result<Response, Self::Error> {
        let ms = self.client.addressbook_query(path, depth, &query).await?;
        Ok(Response::Multistatus(ms))
    }

    async fn addressbook_multiget(
        &self,
        path: &str,
        multiget: AddressbookMultiget,
    ) -> Result<Response, Self::Error> {
        let ms = self.client.addressbook_multiget(path, &multiget).await?;
        Ok(Response::Multistatus(ms))
    }
}
