//! WebDAV report structures, report advertisements, and property expansion models according to RFC 3253.

use crate::webdav::error::DavError;
use crate::webdav::property::{Property, PropertyValue};
use crate::webdav::xml::{
    DAV_NS, Elem, FromXmlDocument, FromXmlElem, Node, QName, ToXmlDocument, ToXmlNode,
    parse_document_root, serialize_document,
};

/// The `DAV:supported-report-set` property value (RFC 3253 §3.1.5).
///
/// DTD:
/// ```xml
/// <!ELEMENT supported-report-set (supported-report*)>
/// <!ELEMENT supported-report report>
/// <!ELEMENT report ANY>
/// ```
///
/// Each entry in `reports` is the QName of a report element type (e.g.
/// `DAV:sync-collection`, `DAV:expand-property`).
#[derive(Clone, Debug, PartialEq)]
pub struct SupportedReportSet {
    pub reports: Vec<QName>,
}

impl ToXmlNode for SupportedReportSet {
    fn to_node(&self) -> Node {
        Node::dav_elem("supported-report-set", self.to_nodes())
    }
}

impl FromXmlElem for SupportedReportSet {
    fn from_elem(elem: &Elem) -> Result<Self, DavError> {
        Ok(Self::from_nodes(&elem.children))
    }
}

impl SupportedReportSet {
    /// Convert to the child nodes of a `D:supported-report-set` property
    /// element.
    pub fn to_nodes(&self) -> Vec<Node> {
        self.reports
            .iter()
            .map(|r| {
                Node::dav_elem(
                    "supported-report",
                    vec![Node::dav_elem(
                        "report",
                        vec![Node::elem(r.clone(), vec![])],
                    )],
                )
            })
            .collect()
    }

    /// Parse from the child nodes of a `D:supported-report-set` property
    /// element.
    pub fn from_nodes(nodes: &[Node]) -> Self {
        let reports = nodes
            .iter()
            .filter_map(|n| match n {
                Node::Elem(e) if e.name.is(DAV_NS, "supported-report") => e
                    .child_elem(DAV_NS, "report")
                    .and_then(|r| r.all_child_elems().next().map(|inner| inner.name.clone())),
                _ => None,
            })
            .collect();
        Self { reports }
    }

    /// Construct a `D:supported-report-set` property with this value.
    pub fn to_property(&self) -> Property {
        Property::new(
            QName::dav("supported-report-set"),
            PropertyValue::Raw(self.to_nodes()),
        )
    }
}

/// A `DAV:expand-property` report request body (RFC 3253 §3.8).
///
/// DTD:
/// ```xml
/// <!ELEMENT expand-property (property*)>
/// <!ELEMENT property (property*)>
/// <!ATTLIST property name NMTOKEN #REQUIRED>
/// <!ATTLIST property namespace NMTOKEN "DAV:">
/// ```
///
/// This report retrieves properties from resources identified by `DAV:href`
/// elements in property values, expanding them inline.
#[derive(Clone, Debug, PartialEq)]
pub struct ExpandProperty {
    pub properties: Vec<ExpandPropertyEntry>,
}

/// A single `D:property` element inside an `expand-property` request.
#[derive(Clone, Debug, PartialEq)]
pub struct ExpandPropertyEntry {
    /// The property to expand (name + namespace).
    pub name: QName,
    /// Nested properties to expand recursively.
    pub nested: Vec<ExpandPropertyEntry>,
}

impl ToXmlNode for ExpandPropertyEntry {
    fn to_node(&self) -> Node {
        let mut elem = Elem::dav("property").with_attr("name", &self.name.local);
        if self.name.namespace != DAV_NS {
            elem = elem.with_attr("namespace", &self.name.namespace);
        }
        elem = elem.with_children(self.nested.iter().map(|n| n.to_node()));
        elem.to_node()
    }
}

impl FromXmlElem for ExpandPropertyEntry {
    fn from_elem(elem: &Elem) -> Result<Self, DavError> {
        let local = elem
            .attributes
            .iter()
            .find(|(k, _)| k == "name")
            .map(|(_, v)| v.clone())
            .ok_or_else(|| DavError::MissingField("D:property requires name attribute".into()))?;
        let namespace = elem
            .attributes
            .iter()
            .find(|(k, _)| k == "namespace")
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| DAV_NS.to_string());
        let nested = elem.parse_children(DAV_NS, "property")?;
        Ok(ExpandPropertyEntry {
            name: QName::new(namespace, local),
            nested,
        })
    }
}

impl ToXmlDocument for ExpandProperty {
    fn to_xml(&self) -> Vec<u8> {
        let children: Vec<Node> = self.properties.iter().map(|p| p.to_node()).collect();
        serialize_document("expand-property", DAV_NS, &children)
    }
}

impl FromXmlElem for ExpandProperty {
    fn from_elem(elem: &Elem) -> Result<Self, DavError> {
        let properties = elem.parse_children(DAV_NS, "property")?;
        Ok(Self { properties })
    }
}

impl FromXmlDocument for ExpandProperty {
    fn from_xml(input: &[u8]) -> Result<Self, DavError> {
        let root = parse_document_root(input, DAV_NS, "expand-property")?;
        Self::from_elem(&root)
    }
}

impl ExpandProperty {
    pub fn to_xml(&self) -> Vec<u8> {
        <Self as ToXmlDocument>::to_xml(self)
    }

    pub fn from_xml(input: &[u8]) -> Result<Self, DavError> {
        <Self as FromXmlDocument>::from_xml(input)
    }
}
