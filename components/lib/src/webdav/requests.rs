//! WebDAV request models for property inspection (PROPFIND) and modification (PROPPATCH) according to RFC 4918.

use crate::webdav::error::DavError;
use crate::webdav::property::{Property, PropertyValue};
use crate::webdav::xml::{
    DAV_NS, Elem, FromXmlDocument, FromXmlElem, Node, QName, ToXmlDocument, parse_document_root,
    serialize_document,
};

/// A `DAV:propfind` request body. Three forms per RFC 4918 §9.1:
/// `allprop`, `propname`, and `prop` (specific properties).
#[derive(Clone, Debug, PartialEq)]
pub enum Propfind {
    /// Return the values of all live + dead properties.
    AllProp,
    /// Return just the names of all properties.
    PropName,
    /// Return values for the requested properties (empty = all properties).
    Prop { properties: Vec<Property> },
}

impl ToXmlDocument for Propfind {
    fn to_xml(&self) -> Vec<u8> {
        serialize_document("propfind", DAV_NS, &self.to_nodes())
    }
}

impl FromXmlDocument for Propfind {
    fn from_xml(input: &[u8]) -> Result<Self, DavError> {
        let root = parse_document_root(input, DAV_NS, "propfind")?;
        Self::from_nodes(&root.children)
    }
}

impl Propfind {
    /// Return the inner content as child nodes (without the `<D:propfind>` wrapper).
    /// Used by CardDAV report bodies that embed `D:allprop`/`D:propname`/`D:prop`
    /// directly inside a report element.
    pub fn to_nodes(&self) -> Vec<Node> {
        match self {
            Propfind::AllProp => vec![Node::empty_dav("allprop")],
            Propfind::PropName => vec![Node::empty_dav("propname")],
            Propfind::Prop { properties } => {
                let props = properties
                    .iter()
                    .map(|p| Node::elem(p.name.clone(), vec![]))
                    .collect();
                vec![Node::dav_elem("prop", props)]
            }
        }
    }

    /// Parse from child nodes (without the `<D:propfind>` wrapper).
    /// Returns `AllProp` if no recognised child is found (matching the
    /// empty-body convention of PROPFIND).
    pub fn from_nodes(children: &[Node]) -> Result<Self, DavError> {
        for child in children {
            match child {
                Node::Elem(e) => {
                    if e.name.is(DAV_NS, "allprop") {
                        return Ok(Propfind::AllProp);
                    }
                    if e.name.is(DAV_NS, "propname") {
                        return Ok(Propfind::PropName);
                    }
                    if e.name.is(DAV_NS, "prop") {
                        let properties = e
                            .all_child_elems()
                            .map(|c| Property::name_only(c.name.clone()))
                            .collect();
                        return Ok(Propfind::Prop { properties });
                    }
                }
                Node::Text(t) => {
                    if !t.trim().is_empty() {
                        return Err(DavError::Invalid(format!(
                            "invalid propfind text content: {}",
                            t
                        )));
                    }
                }
            }
        }
        Ok(Propfind::AllProp)
    }

    pub fn to_xml(&self) -> Vec<u8> {
        <Self as ToXmlDocument>::to_xml(self)
    }

    pub fn from_xml(input: &[u8]) -> Result<Self, DavError> {
        <Self as FromXmlDocument>::from_xml(input)
    }
}

/// A `DAV:propertyupdate` (PROPPATCH) request: a list of sets and removes.
#[derive(Clone, Debug, PartialEq)]
pub struct Proppatch {
    /// Properties to set (with values).
    pub set: Vec<Property>,
    /// Property names to remove.
    pub remove: Vec<QName>,
}

impl FromXmlElem for Proppatch {
    fn from_elem(elem: &Elem) -> Result<Self, DavError> {
        let mut set = Vec::new();
        let mut remove = Vec::new();

        for child in elem.all_child_elems() {
            if child.name.is(DAV_NS, "set") {
                if let Some(prop) = child.child_elem(DAV_NS, "prop") {
                    for pc in prop.all_child_elems() {
                        let value = PropertyValue::from_nodes(&pc.children);
                        set.push(Property::new(pc.name.clone(), value));
                    }
                }
            } else if child.name.is(DAV_NS, "remove")
                && let Some(prop) = child.child_elem(DAV_NS, "prop")
            {
                for pc in prop.all_child_elems() {
                    remove.push(pc.name.clone());
                }
            }
        }
        Ok(Self { set, remove })
    }
}

impl FromXmlDocument for Proppatch {
    fn from_xml(input: &[u8]) -> Result<Self, DavError> {
        let root = parse_document_root(input, DAV_NS, "propertyupdate")?;
        Self::from_elem(&root)
    }
}

impl Proppatch {
    pub fn to_nodes(&self) -> Vec<Node> {
        let mut children = Vec::new();
        if !self.set.is_empty() {
            let props = self
                .set
                .iter()
                .map(|p| Node::elem(p.name.clone(), p.to_nodes()))
                .collect();
            children.push(Node::dav_elem("set", vec![Node::dav_elem("prop", props)]));
        }
        if !self.remove.is_empty() {
            let props = self
                .remove
                .iter()
                .map(|n| Node::elem(n.clone(), vec![]))
                .collect();
            children.push(Node::dav_elem(
                "remove",
                vec![Node::dav_elem("prop", props)],
            ));
        }
        children
    }

    pub fn to_xml(&self) -> Result<Vec<u8>, DavError> {
        if self.set.is_empty() && self.remove.is_empty() {
            return Err(DavError::MissingField(
                "propertyupdate requires at least one set or remove".into(),
            ));
        }
        Ok(serialize_document(
            "propertyupdate",
            DAV_NS,
            &self.to_nodes(),
        ))
    }

    pub fn from_xml(input: &[u8]) -> Result<Self, DavError> {
        <Self as FromXmlDocument>::from_xml(input)
    }
}
