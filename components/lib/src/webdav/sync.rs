//! WebDAV change synchronization request and response models according to RFC 6578.

use crate::webdav::error::DavError;
use crate::webdav::property::{Property, PropertyValue};
use crate::webdav::xml::{
    DAV_NS, Elem, FromXmlDocument, FromXmlElem, Node, QName, ToXmlDocument, parse_document_root,
    serialize_document,
};

/// The scope of a `DAV:sync-collection` report (RFC 6578 §3.3, §6.3).
///
/// `<!ELEMENT sync-level CDATA>`
/// Text MUST be either "1" or "infinite".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyncLevel {
    /// Only immediate children of the collection.
    One,
    /// All descendants at any depth.
    Infinite,
}

impl SyncLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            SyncLevel::One => "1",
            SyncLevel::Infinite => "infinite",
        }
    }

    fn from_str(s: &str) -> Result<Self, DavError> {
        match s {
            "1" => Ok(SyncLevel::One),
            "infinite" => Ok(SyncLevel::Infinite),
            _ => Err(DavError::Invalid(format!("invalid sync-level value: {s}"))),
        }
    }
}

/// A `DAV:sync-collection` report request body (RFC 6578 §6.1).
///
/// DTD: `<!ELEMENT sync-collection (sync-token, sync-level, limit?, prop)>`
///
/// When `sync_token` is `None`, the request is an initial synchronization
/// (empty `<D:sync-token/>` element). When it is `Some(token)`, the server
/// returns only changes since that token.
#[derive(Clone, Debug, PartialEq)]
pub struct SyncCollection {
    /// The sync-token from the last sync, or `None` for initial sync.
    pub sync_token: Option<String>,
    /// The scope of synchronization.
    pub sync_level: SyncLevel,
    /// Property names to return for each changed member.
    pub prop: Vec<Property>,
    /// Optional limit on the number of results (RFC 5323 §5.17 `DAV:nresults`).
    pub limit: Option<u32>,
}

impl ToXmlDocument for SyncCollection {
    fn to_xml(&self) -> Vec<u8> {
        let sync_token_node = match &self.sync_token {
            Some(t) => Node::dav_elem("sync-token", vec![Node::text_node(t.clone())]),
            None => Node::empty_dav("sync-token"),
        };
        let sync_level_node = Node::dav_elem(
            "sync-level",
            vec![Node::text_node(self.sync_level.as_str())],
        );
        let prop_node = Node::dav_elem(
            "prop",
            self.prop
                .iter()
                .map(|p| Node::elem(p.name.clone(), vec![]))
                .collect(),
        );
        let mut children = vec![sync_token_node, sync_level_node];
        if let Some(n) = self.limit {
            children.push(Node::dav_elem(
                "limit",
                vec![Node::dav_elem(
                    "nresults",
                    vec![Node::text_node(n.to_string())],
                )],
            ));
        }
        children.push(prop_node);
        serialize_document("sync-collection", DAV_NS, &children)
    }
}

impl FromXmlElem for SyncCollection {
    fn from_elem(elem: &Elem) -> Result<Self, DavError> {
        let sync_token_elem = elem
            .require_child(DAV_NS, "sync-token")
            .map_err(|_| DavError::MissingField("sync-collection requires D:sync-token".into()))?;
        let sync_token = {
            let text = sync_token_elem.text();
            if text.is_empty() { None } else { Some(text) }
        };
        let sync_level_text = elem
            .require_child_text(DAV_NS, "sync-level")
            .map_err(|_| DavError::MissingField("sync-collection requires D:sync-level".into()))?;
        let sync_level = SyncLevel::from_str(&sync_level_text)?;
        let prop = elem
            .child_elem(DAV_NS, "prop")
            .map(|p| {
                p.all_child_elems()
                    .map(|e| Property::name_only(e.name.clone()))
                    .collect()
            })
            .ok_or_else(|| DavError::MissingField("sync-collection requires D:prop".into()))?;
        let limit = elem
            .child_elem(DAV_NS, "limit")
            .and_then(|l| l.child_text(DAV_NS, "nresults"))
            .and_then(|s| s.parse::<u32>().ok());

        Ok(Self {
            sync_token,
            sync_level,
            prop,
            limit,
        })
    }
}

impl FromXmlDocument for SyncCollection {
    fn from_xml(input: &[u8]) -> Result<Self, DavError> {
        let root = parse_document_root(input, DAV_NS, "sync-collection")?;
        Self::from_elem(&root)
    }
}

impl SyncCollection {
    pub fn to_xml(&self) -> Vec<u8> {
        <Self as ToXmlDocument>::to_xml(self)
    }

    pub fn from_xml(input: &[u8]) -> Result<Self, DavError> {
        <Self as FromXmlDocument>::from_xml(input)
    }
}

/// Construct a `DAV:sync-token` property (RFC 6578 §4) carrying the current
/// sync token value. This is a live property on resources that support
/// sync-collection.
pub fn sync_token_property(token: &str) -> Property {
    Property::new(QName::dav("sync-token"), PropertyValue::Text(token.into()))
}
