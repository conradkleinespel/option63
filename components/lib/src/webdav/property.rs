//! Property data models and typed values shared across WebDAV and CardDAV operations.

use crate::webdav::error::DavError;
use crate::webdav::xml::{DAV_NS, Elem, FromXmlElem, Node, QName, ToXmlNode};

/// A WebDAV property: a qualified name plus a value that is either typed
/// (for modeled properties) or raw XML (for lossless passthrough).
///
/// This is intentionally "semi-open": backends that understand a property
/// model it with a typed variant; backends (such as a proxy) that must
/// round-trip arbitrary properties use the raw form so nothing is lost.
#[derive(Clone, Debug, PartialEq)]
pub struct Property {
    pub name: QName,
    pub value: PropertyValue,
}

/// The value of a [`Property`]. Typed variants model common WebDAV value
/// shapes; [`PropertyValue::Raw`] carries the raw child nodes verbatim.
#[derive(Clone, Debug, PartialEq)]
pub enum PropertyValue {
    /// Empty element, e.g. `<D:resourcetype/>`.
    Empty,
    /// A text value, e.g. `displayname`.
    Text(String),
    /// A single `<D:href>` child, e.g. `principal-URL`.
    Href(String),
    /// Multiple `<D:href>` children, e.g. `addressbook-home-set`.
    Hrefs(Vec<String>),
    /// Arbitrary child nodes (passthrough / unmodeled properties).
    Raw(Vec<Node>),
}

impl PropertyValue {
    /// Classify a child-node slice into a typed value where recognisable,
    /// falling back to [`PropertyValue::Raw`].
    ///
    /// * empty → [`PropertyValue::Empty`]
    /// * single text node → [`PropertyValue::Text`]
    /// * one or more `href` elements → [`PropertyValue::Href`]/[`PropertyValue::Hrefs`]
    /// * anything else → [`PropertyValue::Raw`]
    pub fn from_nodes(nodes: &[Node]) -> PropertyValue {
        match nodes {
            [] => PropertyValue::Empty,
            [Node::Text(t)] => PropertyValue::Text(t.clone()),
            [Node::Elem(e)] if e.name.is(DAV_NS, "href") => PropertyValue::Href(e.text()),
            many if many
                .iter()
                .all(|n| matches!(n, Node::Elem(e) if e.name.is(DAV_NS, "href"))) =>
            {
                PropertyValue::Hrefs(
                    many.iter()
                        .filter_map(|n| match n {
                            Node::Elem(e) => Some(e.text()),
                            _ => None,
                        })
                        .collect(),
                )
            }
            _ => PropertyValue::Raw(nodes.to_vec()),
        }
    }
}

impl ToXmlNode for Property {
    fn to_node(&self) -> Node {
        Node::elem(self.name.clone(), self.to_nodes())
    }
}

impl FromXmlElem for Property {
    fn from_elem(elem: &Elem) -> Result<Self, DavError> {
        Ok(Property::new(
            elem.name.clone(),
            PropertyValue::from_nodes(&elem.children),
        ))
    }
}

impl Property {
    pub fn new(name: QName, value: PropertyValue) -> Self {
        Self { name, value }
    }

    /// A property name-only request (no value).
    pub fn name_only(name: QName) -> Self {
        Self::new(name, PropertyValue::Empty)
    }

    pub fn dav(local: impl Into<String>, value: PropertyValue) -> Self {
        Self::new(QName::dav(local), value)
    }

    pub fn is(&self, namespace: &str, local: &str) -> bool {
        self.name.is(namespace, local)
    }

    /// Convert a property to its XML child node(s).
    pub fn to_nodes(&self) -> Vec<Node> {
        match &self.value {
            PropertyValue::Empty => vec![],
            PropertyValue::Text(t) => vec![Node::text_node(t.clone())],
            PropertyValue::Href(u) => {
                vec![Node::href(u.clone())]
            }
            PropertyValue::Hrefs(urls) => urls.iter().map(|u| Node::href(u.clone())).collect(),
            PropertyValue::Raw(nodes) => nodes.clone(),
        }
    }

    /// Whether this property carries any value content.
    pub fn is_empty_value(&self) -> bool {
        matches!(self.value, PropertyValue::Empty)
    }
}
