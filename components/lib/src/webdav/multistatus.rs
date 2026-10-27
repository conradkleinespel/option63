//! Multi-status (207) response models, individual resource responses, and status serialization according to RFC 4918.

use crate::webdav::error::DavError;
use crate::webdav::property::{Property, PropertyValue};
use crate::webdav::xml::{
    DAV_NS, Elem, FromXmlDocument, FromXmlElem, Node, QName, ToXmlDocument, ToXmlNode,
    parse_document_root, serialize_document,
};

/// A `DAV:multistatus` response body (HTTP 207), used by both PROPFIND and
/// REPORT responses and shared verbatim between the client and server.
///
/// When used in a `DAV:sync-collection` response (RFC 6578), `sync_token`
/// carries the synchronization token the client should use in the next
/// request.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Multistatus {
    pub responses: Vec<PropResponse>,
    /// Optional trailing `D:responsedescription` text.
    pub response_description: Option<String>,
    /// Optional `D:sync-token` (RFC 6578 §6.2), present in sync-collection
    /// responses.
    pub sync_token: Option<String>,
}

/// The payload of a `D:response` element: either propstats or a resource-level status.
#[derive(Clone, Debug, PartialEq)]
pub enum PropResponseKind {
    /// One or more propstat groups for this resource.
    Propstats(Vec<Propstat>),
    /// A resource-level status (for non-propstat responses, e.g. reports).
    Status(Status),
}

/// A single `D:response` inside a multistatus.
#[derive(Clone, Debug, PartialEq)]
pub struct PropResponse {
    pub href: String,
    pub kind: PropResponseKind,
}

impl PropResponse {
    pub fn propstat(href: impl Into<String>, propstat: Propstat) -> Self {
        Self {
            href: href.into(),
            kind: PropResponseKind::Propstats(vec![propstat]),
        }
    }

    pub fn with_propstats(href: impl Into<String>, propstats: Vec<Propstat>) -> Self {
        Self {
            href: href.into(),
            kind: PropResponseKind::Propstats(propstats),
        }
    }

    pub fn with_status(href: impl Into<String>, status: Status) -> Self {
        Self {
            href: href.into(),
            kind: PropResponseKind::Status(status),
        }
    }

    pub fn propstats(&self) -> Option<&[Propstat]> {
        match &self.kind {
            PropResponseKind::Propstats(ps) => Some(ps.as_slice()),
            PropResponseKind::Status(_) => None,
        }
    }

    pub fn propstats_mut(&mut self) -> Option<&mut Vec<Propstat>> {
        match &mut self.kind {
            PropResponseKind::Propstats(ps) => Some(ps),
            PropResponseKind::Status(_) => None,
        }
    }

    pub fn status(&self) -> Option<&Status> {
        match &self.kind {
            PropResponseKind::Status(s) => Some(s),
            PropResponseKind::Propstats(_) => None,
        }
    }
}

/// A `D:propstat`: a set of properties sharing the same status.
#[derive(Clone, Debug, PartialEq)]
pub struct Propstat {
    pub prop: Vec<Property>,
    pub status: Status,
    /// Optional per-propstat description.
    pub response_description: Option<String>,
}

/// An HTTP status line embedded in a propstat/response, e.g. `HTTP/1.1 200 OK`.
#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    pub code: u16,
    pub reason: String,
}

impl Status {
    pub fn new(code: u16, reason: impl Into<String>) -> Self {
        Self {
            code,
            reason: reason.into(),
        }
    }

    pub fn ok() -> Self {
        Self::new(200, "OK")
    }

    pub fn not_found() -> Self {
        Self::new(404, "Not Found")
    }

    pub fn forbidden() -> Self {
        Self::new(403, "Forbidden")
    }
}

impl ToXmlDocument for Multistatus {
    fn to_xml(&self) -> Vec<u8> {
        let children: Vec<Node> =
            self.responses
                .iter()
                .map(|r| r.to_node())
                .chain(self.response_description.iter().map(|d| {
                    Node::dav_elem("responsedescription", vec![Node::text_node(d.clone())])
                }))
                .chain(
                    self.sync_token
                        .iter()
                        .map(|t| Node::dav_elem("sync-token", vec![Node::text_node(t.clone())])),
                )
                .collect();
        serialize_document("multistatus", DAV_NS, &children)
    }
}

impl FromXmlDocument for Multistatus {
    fn from_xml(input: &[u8]) -> Result<Self, DavError> {
        let root = parse_document_root(input, DAV_NS, "multistatus")?;
        let responses = root.parse_children(DAV_NS, "response")?;
        let response_description = root.child_text(DAV_NS, "responsedescription");
        let sync_token = root.child_text(DAV_NS, "sync-token");
        Ok(Self {
            responses,
            response_description,
            sync_token,
        })
    }
}

impl Multistatus {
    pub fn to_xml(&self) -> Vec<u8> {
        <Self as ToXmlDocument>::to_xml(self)
    }

    pub fn from_xml(input: &[u8]) -> Result<Self, DavError> {
        <Self as FromXmlDocument>::from_xml(input)
    }
}

impl ToXmlNode for PropResponse {
    fn to_node(&self) -> Node {
        let mut elem = Elem::dav("response").with_child(Node::href(self.href.clone()));
        match &self.kind {
            PropResponseKind::Propstats(propstats) => {
                elem = elem.with_children(propstats.iter().map(|ps| ps.to_node()));
            }
            PropResponseKind::Status(status) => {
                elem = elem.with_child(status_node(status));
            }
        }
        elem.to_node()
    }
}

impl FromXmlElem for PropResponse {
    fn from_elem(elem: &Elem) -> Result<Self, DavError> {
        let href = elem.require_child_text(DAV_NS, "href")?;
        let propstats = elem.parse_children(DAV_NS, "propstat")?;
        let status = elem
            .child_text(DAV_NS, "status")
            .map(|s| parse_status(&s))
            .transpose()?;

        let kind = match (propstats.is_empty(), status) {
            (false, None) => PropResponseKind::Propstats(propstats),
            (true, Some(status)) => PropResponseKind::Status(status),
            (false, Some(_)) => {
                return Err(DavError::Invalid(
                    "invalid response, cannot have a status with propstats".to_string(),
                ));
            }
            (true, None) => {
                return Err(DavError::Invalid(
                    "invalid response, has neither propstats nor status".to_string(),
                ));
            }
        };

        Ok(Self { href, kind })
    }
}

impl PropResponse {
    pub fn to_node(&self) -> Node {
        <Self as ToXmlNode>::to_node(self)
    }

    pub fn from_elem(elem: &Elem) -> Result<Self, DavError> {
        <Self as FromXmlElem>::from_elem(elem)
    }
}

impl ToXmlNode for Propstat {
    fn to_node(&self) -> Node {
        let prop_children = self
            .prop
            .iter()
            .map(|p| Node::elem(p.name.clone(), p.to_nodes()))
            .collect();
        let mut elem = Elem::dav("propstat")
            .with_child(Node::dav_elem("prop", prop_children))
            .with_child(status_node(&self.status));
        if let Some(desc) = &self.response_description {
            elem = elem.with_child(Node::dav_elem(
                "responsedescription",
                vec![Node::text_node(desc.clone())],
            ));
        }
        elem.to_node()
    }
}

impl FromXmlElem for Propstat {
    fn from_elem(elem: &Elem) -> Result<Self, DavError> {
        let prop_elem = elem.require_child(DAV_NS, "prop")?;
        let status_str = elem.require_child_text(DAV_NS, "status")?;
        let properties = prop_elem
            .all_child_elems()
            .map(|p| {
                let name = p.name.clone();
                let value = PropertyValue::from_nodes(&p.children);
                Property::new(name, value)
            })
            .collect();
        let response_description = elem.child_text(DAV_NS, "responsedescription");
        Ok(Self {
            prop: properties,
            status: parse_status(&status_str)?,
            response_description,
        })
    }
}

impl Propstat {
    pub fn to_node(&self) -> Node {
        <Self as ToXmlNode>::to_node(self)
    }

    pub fn from_elem(elem: &Elem) -> Result<Self, DavError> {
        <Self as FromXmlElem>::from_elem(elem)
    }
}

fn status_node(status: &Status) -> Node {
    Node::elem(
        QName::dav("status"),
        vec![Node::text_node(format!(
            "HTTP/1.1 {} {}",
            status.code, status.reason
        ))],
    )
}

fn parse_status(text: &str) -> Result<Status, DavError> {
    // Status line: HTTP/1.1 200 OK
    let parts: Vec<&str> = text.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(DavError::XmlParse(format!("malformed status in `{text}`")));
    }
    let code: u16 = parts[1]
        .parse()
        .map_err(|_| DavError::XmlParse(format!("malformed status code in `{text}`")))?;
    let reason = parts[2..].join(" ");
    Ok(Status { code, reason })
}
