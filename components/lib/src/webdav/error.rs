//! Error types for webdav servers and clients, as well as their XML representation.

use crate::webdav::{
    DAV_NS, Elem, FromXmlDocument, FromXmlElem, Node, QName, ToXmlDocument, ToXmlNode,
    parse_document_root, serialize_document,
};
use quick_xml::events::attributes::AttrError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DavError {
    #[error("invalid XML: {0}")]
    XmlParse(String),
    #[error("failed to serialize XML: {0}")]
    XmlSerialize(#[source] std::io::Error),
    #[error("XML attribute error: {0}")]
    XmlAttr(#[source] AttrError),
    #[error("malformed WWW-Authenticate challenge: {0}")]
    InvalidChallenge(String),
    #[error("unsupported content type `{0}`")]
    UnsupportedContentType(String),
    #[error("missing required field: {0}")]
    MissingField(String),
    #[error("HTTP request failed: {0}")]
    Http(#[source] reqwest::Error),
    #[error("unexpected HTTP status `{0}`")]
    HttpStatus(u16),
    #[error("{0}")]
    Invalid(String),
}

/// A `DAV:error` response body: an optional precondition element plus an
/// optional human-readable description (RFC 4918 §13).
#[derive(Clone, Debug, PartialEq)]
pub struct DavErrorBody {
    /// The precondition/tag element, e.g. `D:cannot-modify-protected-property`.
    /// Stored as a qualified name; typically within the `DAV:` namespace, but
    /// may be in an extension namespace.
    pub condition: Option<QName>,
    /// Human-readable `D:responsedescription` text.
    pub response_description: Option<String>,
}

impl ToXmlNode for DavErrorBody {
    fn to_node(&self) -> Node {
        let condition = self
            .condition
            .clone()
            .unwrap_or_else(|| QName::dav("server-error"));
        let mut elem = Elem::dav("error").with_child(Node::empty(condition));
        if let Some(desc) = &self.response_description {
            elem = elem.with_child(Node::dav_elem(
                "responsedescription",
                vec![Node::text_node(desc.clone())],
            ));
        }
        elem.to_node()
    }
}

impl FromXmlElem for DavErrorBody {
    fn from_elem(elem: &Elem) -> Result<Self, DavError> {
        let condition = elem
            .all_child_elems()
            .find(|e| !e.name.is(DAV_NS, "responsedescription"))
            .map(|e| e.name.clone());
        let response_description = elem.child_text(DAV_NS, "responsedescription");
        Ok(Self {
            condition,
            response_description,
        })
    }
}

impl ToXmlDocument for DavErrorBody {
    fn to_xml(&self) -> Vec<u8> {
        let condition = self
            .condition
            .clone()
            .unwrap_or_else(|| QName::dav("server-error"));
        let mut children = vec![Node::empty(condition)];
        if let Some(desc) = &self.response_description {
            children.push(Node::dav_elem(
                "responsedescription",
                vec![Node::text_node(desc.clone())],
            ));
        }
        serialize_document("error", DAV_NS, &children)
    }
}

impl FromXmlDocument for DavErrorBody {
    fn from_xml(input: &[u8]) -> Result<Self, DavError> {
        let root = parse_document_root(input, DAV_NS, "error")?;
        Self::from_elem(&root)
    }
}

impl DavErrorBody {
    pub fn to_xml(&self) -> Vec<u8> {
        <Self as ToXmlDocument>::to_xml(self)
    }

    pub fn from_xml(input: &[u8]) -> Result<Self, DavError> {
        <Self as FromXmlDocument>::from_xml(input)
    }
}

/// Create a `DAV:error` body for a known DAV precondition.
pub fn dav_error(condition: &str, response_description: impl Into<String>) -> DavErrorBody {
    DavErrorBody {
        condition: Some(QName::dav(condition)),
        response_description: Some(response_description.into()),
    }
}
