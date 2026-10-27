//! CardDAV client implementation for address book discovery, report queries, synchronization, and card management.

use crate::carddav::protocol::{
    AddressbookInfo, AddressbookMultiget, AddressbookQuery, CARDDAV_NS,
};
use crate::webdav::client::{Credentials, DavClient};
use crate::webdav::error::DavError;
use crate::webdav::resource::Resource;
use crate::webdav::{
    DAV_NS, Depth, Multistatus, Property, PropertyValue, Propfind, QName, SyncCollection,
};

/// A CardDAV client built on top of [`DavClient`].
///
/// Provides CardDAV-specific discovery, reports, and card CRUD operations
/// using the shared WebDAV protocol types.
#[derive(Clone)]
pub struct CarddavClient {
    dav: DavClient,
}

impl CarddavClient {
    pub fn new(dav: DavClient) -> Self {
        Self { dav }
    }

    pub fn with_credentials(base: reqwest::Url, creds: Credentials) -> Self {
        Self::new(DavClient::new(base).with_credentials(creds))
    }

    /// Discovery step 1: find the current user's principal URL (RFC 5397).
    ///
    /// PROPFIND on the root for `D:current-user-principal`.
    pub async fn current_user_principal(&self) -> Result<Option<String>, DavError> {
        let propfind = Propfind::Prop {
            properties: vec![Property::name_only(QName::dav("current-user-principal"))],
        };
        let ms = self.dav.propfind("/", Depth::Zero, &propfind).await?;
        let href = ms
            .responses
            .first()
            .and_then(|r| r.propstats())
            .and_then(|ps| ps.first())
            .and_then(|ps| {
                ps.prop
                    .iter()
                    .find(|p| p.is(DAV_NS, "current-user-principal"))
            })
            .and_then(|p| match &p.value {
                PropertyValue::Href(h) => Some(h.clone()),
                PropertyValue::Hrefs(hs) => hs.first().cloned(),
                PropertyValue::Raw(nodes) => {
                    for node in nodes {
                        if let crate::webdav::Node::Elem(e) = node
                            && e.name.is(DAV_NS, "href")
                        {
                            return Some(e.text());
                        }
                    }
                    None
                }
                _ => None,
            });
        Ok(href)
    }

    /// Discovery step 2: find the address book home set URL(s) (RFC 6352 §7.1.1).
    ///
    /// PROPFIND on the principal URL for `C:addressbook-home-set`.
    pub async fn addressbook_home_set(&self, principal: &str) -> Result<Vec<String>, DavError> {
        let propfind = Propfind::Prop {
            properties: vec![Property::name_only(QName::new(
                CARDDAV_NS,
                "addressbook-home-set",
            ))],
        };
        let ms = self.dav.propfind(principal, Depth::Zero, &propfind).await?;
        let hrefs = ms
            .responses
            .first()
            .and_then(|r| r.propstats())
            .and_then(|ps| ps.first())
            .and_then(|ps| {
                ps.prop
                    .iter()
                    .find(|p| p.is(CARDDAV_NS, "addressbook-home-set"))
            })
            .and_then(|p| match &p.value {
                PropertyValue::Hrefs(hs) => Some(hs.clone()),
                PropertyValue::Href(h) => Some(vec![h.clone()]),
                PropertyValue::Raw(nodes) => {
                    let mut hs = Vec::new();
                    for node in nodes {
                        if let crate::webdav::Node::Elem(e) = node
                            && e.name.is(DAV_NS, "href")
                        {
                            hs.push(e.text());
                        }
                    }
                    if hs.is_empty() { None } else { Some(hs) }
                }
                _ => None,
            });
        Ok(hrefs.unwrap_or_default())
    }

    /// Discovery step 3: find address book collections in the home set.
    ///
    /// PROPFIND Depth:1 on the home set, looking for resources with
    /// `C:addressbook` in their resourcetype.
    pub async fn find_addressbooks(
        &self,
        home_set: &str,
    ) -> Result<Vec<AddressbookInfo>, DavError> {
        let propfind = Propfind::Prop {
            properties: vec![
                Property::name_only(QName::dav("resourcetype")),
                Property::name_only(QName::new(CARDDAV_NS, "addressbook-description")),
            ],
        };
        let ms = self.dav.propfind(home_set, Depth::One, &propfind).await?;

        let addressbooks = ms
            .responses
            .iter()
            .filter_map(|r| {
                let ps = r.propstats()?.first()?;
                let has_addressbook = ps.prop.iter().any(|p| {
                    if p.is(DAV_NS, "resourcetype") {
                        match &p.value {
                            PropertyValue::Raw(nodes) => nodes.iter().any(|n| {
                                matches!(n, crate::webdav::Node::Elem(e)
                                    if e.name.is(CARDDAV_NS, "addressbook"))
                            }),
                            _ => false,
                        }
                    } else {
                        false
                    }
                });
                if !has_addressbook {
                    return None;
                }
                let description = ps.prop.iter().find_map(|p| {
                    if p.is(CARDDAV_NS, "addressbook-description") {
                        match &p.value {
                            PropertyValue::Text(t) => Some(t.clone()),
                            _ => None,
                        }
                    } else {
                        None
                    }
                });
                Some(AddressbookInfo {
                    href: r.href.clone(),
                    description,
                })
            })
            .collect();

        Ok(addressbooks)
    }

    /// Send a `C:addressbook-query` report (RFC 6352 §8.6).
    pub async fn addressbook_query(
        &self,
        path: &str,
        depth: Depth,
        query: &AddressbookQuery,
    ) -> Result<Multistatus, DavError> {
        self.dav.report(path, depth, &query.to_xml()).await
    }

    /// Send a `C:addressbook-multiget` report (RFC 6352 §8.7).
    ///
    /// Uses `Depth: 1` per the RFC examples (the scope is determined by the
    /// `D:href` elements in the request body, not the Depth header).
    pub async fn addressbook_multiget(
        &self,
        path: &str,
        multiget: &AddressbookMultiget,
    ) -> Result<Multistatus, DavError> {
        self.dav.report(path, Depth::One, &multiget.to_xml()).await
    }

    /// Send a `DAV:sync-collection` report (RFC 6578).
    pub async fn sync_collection(
        &self,
        path: &str,
        request: &SyncCollection,
    ) -> Result<Multistatus, DavError> {
        self.dav.sync_collection(path, request).await
    }

    /// GET an address object resource.
    pub async fn get_card(&self, path: &str) -> Result<Resource, DavError> {
        self.dav.get(path).await
    }

    /// PUT an address object resource.
    pub async fn put_card(&self, path: &str, body: &[u8]) -> Result<u16, DavError> {
        let resource = Resource {
            content_type: "text/vcard".into(),
            etag: None,
            body: body.to_vec(),
        };
        self.dav.put(path, &resource).await
    }

    /// DELETE an address object resource.
    pub async fn delete_card(&self, path: &str) -> Result<u16, DavError> {
        self.dav.delete(path).await
    }

    /// MKCOL a new address book collection.
    pub async fn mkcol_addressbook(&self, path: &str) -> Result<u16, DavError> {
        self.dav.mkcol(path).await
    }

    /// Send an OPTIONS request and return the `DAV` header value.
    pub async fn options(&self, path: &str) -> Result<String, DavError> {
        self.dav.options(path).await
    }

    /// Get a reference to the underlying `DavClient`.
    pub fn dav(&self) -> &DavClient {
        &self.dav
    }
}
