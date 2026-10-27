//! WebDAV Access Control report requests and property search models according to RFC 3744.

use crate::webdav::error::DavError;
use crate::webdav::property::Property;
use crate::webdav::xml::{
    DAV_NS, Elem, FromXmlDocument, FromXmlElem, Node, QName, ToXmlDocument, ToXmlNode,
    parse_document_root, serialize_document,
};

/// A `DAV:acl-principal-prop-set` report request body (RFC 3744 §9.2).
///
/// `<!ELEMENT acl-principal-prop-set ANY>`
///
/// Despite the `ANY` declaration, the marshalling constrains it to a
/// sequence of elements with at most one `D:prop` element.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AclPrincipalPropSet {
    /// Property names to request for each principal.
    pub prop: Vec<Property>,
}

impl ToXmlDocument for AclPrincipalPropSet {
    fn to_xml(&self) -> Vec<u8> {
        let mut children = Vec::new();
        if !self.prop.is_empty() {
            children.push(Node::dav_elem(
                "prop",
                self.prop
                    .iter()
                    .map(|p| Node::elem(p.name.clone(), vec![]))
                    .collect(),
            ));
        }
        serialize_document("acl-principal-prop-set", DAV_NS, &children)
    }
}

impl FromXmlElem for AclPrincipalPropSet {
    fn from_elem(elem: &Elem) -> Result<Self, DavError> {
        // TODO: What does the RFC say about D:acl-principal-prop-set with multiple D:prop children
        let prop = elem
            .child_elem(DAV_NS, "prop")
            .map(|p| {
                p.all_child_elems()
                    .map(|e| Property::name_only(e.name.clone()))
                    .collect()
            })
            .unwrap_or_default();
        Ok(Self { prop })
    }
}

impl FromXmlDocument for AclPrincipalPropSet {
    fn from_xml(input: &[u8]) -> Result<Self, DavError> {
        let root = parse_document_root(input, DAV_NS, "acl-principal-prop-set")?;
        Self::from_elem(&root)
    }
}

impl AclPrincipalPropSet {
    pub fn to_xml(&self) -> Vec<u8> {
        <Self as ToXmlDocument>::to_xml(self)
    }

    pub fn from_xml(input: &[u8]) -> Result<Self, DavError> {
        <Self as FromXmlDocument>::from_xml(input)
    }
}

/// The principal-match criteria (RFC 3744 §9.3).
///
/// TODO: How is this handled: This report is only defined when the Depth header has value "0";
///       other values result in a 400 (Bad Request) error response.
///
/// `<!ELEMENT principal-match ((principal-property | self), prop?)>`
#[derive(Clone, Debug, PartialEq)]
pub struct PrincipalMatch {
    /// Either match by a property's principal href, or by self.
    pub match_type: PrincipalMatchType,
    /// Properties to return for each matching principal.
    pub prop: Vec<Property>,
}

/// The match mode for `principal-match`.
#[derive(Clone, Debug, PartialEq)]
pub enum PrincipalMatchType {
    /// `<!ELEMENT principal-property ANY>` - match principals identified by
    /// the href in the named property (e.g. `D:owner`).
    PrincipalProperty(QName),
    /// `<!ELEMENT self EMPTY>` - match the current user.
    Self_,
}

impl ToXmlDocument for PrincipalMatch {
    fn to_xml(&self) -> Vec<u8> {
        let mut children = vec![match &self.match_type {
            PrincipalMatchType::PrincipalProperty(name) => {
                Node::dav_elem("principal-property", vec![Node::elem(name.clone(), vec![])])
            }
            PrincipalMatchType::Self_ => Node::empty_dav("self"),
        }];
        if !self.prop.is_empty() {
            children.push(Node::dav_elem(
                "prop",
                self.prop
                    .iter()
                    .map(|p| Node::elem(p.name.clone(), vec![]))
                    .collect(),
            ));
        }
        serialize_document("principal-match", DAV_NS, &children)
    }
}

impl FromXmlElem for PrincipalMatch {
    fn from_elem(elem: &Elem) -> Result<Self, DavError> {
        let match_type = if let Some(pp) = elem.child_elem(DAV_NS, "principal-property") {
            let inner = pp.all_child_elems().next().map(|e| e.name.clone());
            match inner {
                Some(name) => PrincipalMatchType::PrincipalProperty(name),
                None => {
                    return Err(DavError::XmlParse(
                        "D:principal-property must contain a child element".into(),
                    ));
                }
            }
        } else if elem.has_child(DAV_NS, "self") {
            PrincipalMatchType::Self_
        } else {
            return Err(DavError::MissingField(
                "D:principal-match requires D:principal-property or D:self".into(),
            ));
        };
        let prop = elem
            .child_elem(DAV_NS, "prop")
            .map(|p| {
                p.all_child_elems()
                    .map(|e| Property::name_only(e.name.clone()))
                    .collect()
            })
            .unwrap_or_default();
        Ok(Self { match_type, prop })
    }
}

impl FromXmlDocument for PrincipalMatch {
    fn from_xml(input: &[u8]) -> Result<Self, DavError> {
        let root = parse_document_root(input, DAV_NS, "principal-match")?;
        Self::from_elem(&root)
    }
}

impl PrincipalMatch {
    pub fn to_xml(&self) -> Vec<u8> {
        <Self as ToXmlDocument>::to_xml(self)
    }

    pub fn from_xml(input: &[u8]) -> Result<Self, DavError> {
        <Self as FromXmlDocument>::from_xml(input)
    }
}

/// A `DAV:principal-property-search` report request body (RFC 3744 §9.4).
///
/// `<!ELEMENT principal-property-search
///     ((property-search+), prop?, apply-to-principal-collection-set?)>`
#[derive(Clone, Debug, PartialEq)]
pub struct PrincipalPropertySearch {
    /// One or more search criteria (ANDed together).
    pub searches: Vec<PropertySearch>,
    /// Properties to return for each matching principal.
    pub prop: Vec<Property>,
    /// If true, search the collections in `D:principal-collection-set`
    /// instead of the request-URI.
    pub apply_to_principal_collection_set: bool,
}

/// A single search criterion (RFC 3744 §9.4).
///
/// `<!ELEMENT property-search (prop, match)>`
/// `<!ELEMENT match #PCDATA>`
#[derive(Clone, Debug, PartialEq)]
pub struct PropertySearch {
    /// Properties to search within.
    pub prop: Vec<Property>,
    /// The search string.
    pub match_: String,
}

impl ToXmlNode for PropertySearch {
    fn to_node(&self) -> Node {
        let prop_node = Node::dav_elem(
            "prop",
            self.prop
                .iter()
                .map(|p| Node::elem(p.name.clone(), vec![]))
                .collect(),
        );
        let match_node = Node::dav_elem("match", vec![Node::text_node(self.match_.clone())]);
        Node::dav_elem("property-search", vec![prop_node, match_node])
    }
}

impl FromXmlElem for PropertySearch {
    fn from_elem(elem: &Elem) -> Result<Self, DavError> {
        let prop = elem
            .require_child(DAV_NS, "prop")
            .map_err(|_| DavError::MissingField("D:property-search requires D:prop".into()))?
            .all_child_elems()
            .map(|inner| Property::name_only(inner.name.clone()))
            .collect();
        let match_ = elem
            .require_child_text(DAV_NS, "match")
            .map_err(|_| DavError::MissingField("D:property-search requires D:match".into()))?;
        Ok(PropertySearch { prop, match_ })
    }
}

impl ToXmlDocument for PrincipalPropertySearch {
    fn to_xml(&self) -> Vec<u8> {
        let mut children: Vec<Node> = self.searches.iter().map(|s| s.to_node()).collect();
        if !self.prop.is_empty() {
            children.push(Node::dav_elem(
                "prop",
                self.prop
                    .iter()
                    .map(|p| Node::elem(p.name.clone(), vec![]))
                    .collect(),
            ));
        }
        if self.apply_to_principal_collection_set {
            children.push(Node::empty_dav("apply-to-principal-collection-set"));
        }
        serialize_document("principal-property-search", DAV_NS, &children)
    }
}

impl FromXmlElem for PrincipalPropertySearch {
    fn from_elem(elem: &Elem) -> Result<Self, DavError> {
        let searches = elem.parse_children(DAV_NS, "property-search")?;
        if searches.is_empty() {
            return Err(DavError::MissingField(
                "D:principal-property-search requires at least one D:property-search".into(),
            ));
        }
        let prop = elem
            .child_elem(DAV_NS, "prop")
            .map(|p| {
                p.all_child_elems()
                    .map(|e| Property::name_only(e.name.clone()))
                    .collect()
            })
            .unwrap_or_default();
        let apply_to_principal_collection_set =
            elem.has_child(DAV_NS, "apply-to-principal-collection-set");
        Ok(Self {
            searches,
            prop,
            apply_to_principal_collection_set,
        })
    }
}

impl FromXmlDocument for PrincipalPropertySearch {
    fn from_xml(input: &[u8]) -> Result<Self, DavError> {
        let root = parse_document_root(input, DAV_NS, "principal-property-search")?;
        Self::from_elem(&root)
    }
}

impl PrincipalPropertySearch {
    pub fn to_xml(&self) -> Vec<u8> {
        <Self as ToXmlDocument>::to_xml(self)
    }

    pub fn from_xml(input: &[u8]) -> Result<Self, DavError> {
        <Self as FromXmlDocument>::from_xml(input)
    }
}

/// The `DAV:principal-search-property-set` report response (RFC 3744 §9.5).
///
/// `<!ELEMENT principal-search-property-set (principal-search-property*)>`
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PrincipalSearchPropertySet {
    pub properties: Vec<PrincipalSearchProperty>,
}

/// A single searchable property (RFC 3744 §9.5).
///
/// `<!ELEMENT principal-search-property (prop, description)>`
/// `<!ELEMENT description #PCDATA>`
#[derive(Clone, Debug, PartialEq)]
pub struct PrincipalSearchProperty {
    /// The searchable property name.
    pub prop: QName,
    /// Human-readable description.
    pub description: String,
}

impl ToXmlNode for PrincipalSearchProperty {
    fn to_node(&self) -> Node {
        let prop_node = Node::dav_elem("prop", vec![Node::elem(self.prop.clone(), vec![])]);
        let desc_elem = Elem::dav("description")
            .with_attr("xml:lang", "en")
            .with_text(self.description.clone());
        Node::dav_elem(
            "principal-search-property",
            vec![prop_node, desc_elem.to_node()],
        )
    }
}

impl FromXmlElem for PrincipalSearchProperty {
    fn from_elem(elem: &Elem) -> Result<Self, DavError> {
        let prop_elem = elem.require_child(DAV_NS, "prop").map_err(|_| {
            DavError::MissingField("D:principal-search-property requires D:prop".into())
        })?;
        let prop = prop_elem
            .all_child_elems()
            .next()
            .map(|inner| inner.name.clone())
            .ok_or_else(|| DavError::MissingField("D:prop must contain a child element".into()))?;
        let description = elem.child_text(DAV_NS, "description").unwrap_or_default();
        Ok(PrincipalSearchProperty { prop, description })
    }
}

impl ToXmlDocument for PrincipalSearchPropertySet {
    fn to_xml(&self) -> Vec<u8> {
        let children: Vec<Node> = self.properties.iter().map(|p| p.to_node()).collect();
        serialize_document("principal-search-property-set", DAV_NS, &children)
    }
}

impl FromXmlElem for PrincipalSearchPropertySet {
    fn from_elem(elem: &Elem) -> Result<Self, DavError> {
        let properties = elem.parse_children(DAV_NS, "principal-search-property")?;
        Ok(Self { properties })
    }
}

impl FromXmlDocument for PrincipalSearchPropertySet {
    fn from_xml(input: &[u8]) -> Result<Self, DavError> {
        let root = parse_document_root(input, DAV_NS, "principal-search-property-set")?;
        Self::from_elem(&root)
    }
}

impl PrincipalSearchPropertySet {
    pub fn to_xml(&self) -> Vec<u8> {
        <Self as ToXmlDocument>::to_xml(self)
    }

    pub fn from_xml(input: &[u8]) -> Result<Self, DavError> {
        <Self as FromXmlDocument>::from_xml(input)
    }
}
