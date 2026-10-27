//! CardDAV XML protocol definitions, report structures, and parsing logic according to RFC 6352.

use crate::webdav::error::DavError;
use crate::webdav::property::{Property, PropertyValue};
use crate::webdav::requests::Propfind;
use crate::webdav::xml::{DAV_NS, Elem, Node, QName, parse_document, serialize_document};

/// The CardDAV XML namespace (RFC 6352 §14.1).
pub const CARDDAV_NS: &str = "urn:ietf:params:xml:ns:carddav";

/// The default collation when none is specified (RFC 6352 §8.3).
pub const DEFAULT_COLLATION: &str = "i;unicode-casemap";

/// The REQUIRED collation (RFC 6352 §8.3).
pub const COLLATION_ASCII_CASEMAP: &str = "i;ascii-casemap";

/// The other REQUIRED collation (RFC 6352 §8.3).
pub const COLLATION_UNICODE_CASEMAP: &str = "i;unicode-casemap";

/// `<C:addressbook/>` element for use inside `<D:resourcetype>` (RFC 6352 §5.2).
/// Returns a `Node` (not a `Property`) because it is a child of `resourcetype`.
pub fn addressbook_element() -> Node {
    Node::elem(QName::new(CARDDAV_NS, "addressbook"), vec![])
}

/// `C:addressbook-description` property (RFC 6352 §6.2.1).
pub fn addressbook_description_property(desc: &str) -> Property {
    Property::new(
        QName::new(CARDDAV_NS, "addressbook-description"),
        PropertyValue::Text(desc.into()),
    )
}

/// `C:addressbook-home-set` property (RFC 6352 §7.1.1).
pub fn addressbook_home_set_property(hrefs: &[String]) -> Property {
    Property::new(
        QName::new(CARDDAV_NS, "addressbook-home-set"),
        PropertyValue::Hrefs(hrefs.to_vec()),
    )
}

/// `C:principal-address` property (RFC 6352 §7.1.2).
pub fn principal_address_property(href: &str) -> Property {
    Property::new(
        QName::new(CARDDAV_NS, "principal-address"),
        PropertyValue::Href(href.into()),
    )
}

/// `C:max-resource-size` property (RFC 6352 §6.2.3).
pub fn max_resource_size_property(size: u64) -> Property {
    Property::new(
        QName::new(CARDDAV_NS, "max-resource-size"),
        PropertyValue::Text(size.to_string()),
    )
}

/// `C:supported-address-data` property (RFC 6352 §6.2.2).
pub fn supported_address_data_property(types: &[AddressDataType]) -> Property {
    let children: Vec<Node> = types
        .iter()
        .map(|t| {
            Node::Elem(Elem {
                name: QName::new(CARDDAV_NS, "address-data-type"),
                attributes: vec![
                    ("content-type".into(), t.content_type.clone()),
                    ("version".into(), t.version.clone()),
                ],
                children: vec![],
            })
        })
        .collect();
    Property::new(
        QName::new(CARDDAV_NS, "supported-address-data"),
        PropertyValue::Raw(children),
    )
}

/// `C:supported-collation-set` property (RFC 6352 §8.3.1).
pub fn supported_collation_set_property(collations: &[String]) -> Property {
    let children: Vec<Node> = collations
        .iter()
        .map(|c| {
            Node::elem(
                QName::new(CARDDAV_NS, "supported-collation"),
                vec![Node::text_node(c.clone())],
            )
        })
        .collect();
    Property::new(
        QName::new(CARDDAV_NS, "supported-collation-set"),
        PropertyValue::Raw(children),
    )
}

/// `C:address-data` response property - carries raw vCard text (RFC 6352 §10.4).
pub fn address_data_response(data: &str) -> Property {
    Property::new(
        QName::new(CARDDAV_NS, "address-data"),
        PropertyValue::Text(data.into()),
    )
}

/// A supported media type for address object resources (RFC 6352 §6.2.2).
#[derive(Clone, Debug, PartialEq)]
pub struct AddressDataType {
    pub content_type: String,
    pub version: String,
}

impl AddressDataType {
    pub fn new(content_type: &str, version: &str) -> Self {
        Self {
            content_type: content_type.into(),
            version: version.into(),
        }
    }

    pub fn vcard3() -> Self {
        Self::new("text/vcard", "3.0")
    }

    pub fn vcard4() -> Self {
        Self::new("text/vcard", "4.0")
    }
}

/// The `C:address-data` element as it appears in a report *request*, specifying
/// which vCard properties to return (RFC 6352 §10.4).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AddressDataRequest {
    pub content_type: String,
    pub version: String,
    pub allprop: bool,
    pub props: Vec<AddressDataProp>,
}

/// A `C:prop` child of `C:address-data` (RFC 6352 §10.4.2).
#[derive(Clone, Debug, PartialEq)]
pub struct AddressDataProp {
    pub name: String,
    pub novalue: bool,
}

impl AddressDataRequest {
    fn from_elem(e: &Elem) -> Self {
        let content_type = e
            .attributes
            .iter()
            .find(|(k, _)| k == "content-type")
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| "text/vcard".into());
        let version = e
            .attributes
            .iter()
            .find(|(k, _)| k == "version")
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| "3.0".into());

        let mut allprop = false;
        let mut props = Vec::new();

        for child in &e.children {
            if let Node::Elem(ce) = child {
                if ce.name.is(CARDDAV_NS, "allprop") {
                    allprop = true;
                } else if ce.name.is(CARDDAV_NS, "prop") {
                    let name = ce
                        .attributes
                        .iter()
                        .find(|(k, _)| k == "name")
                        .map(|(_, v)| v.clone())
                        .unwrap_or_default();
                    let novalue = ce
                        .attributes
                        .iter()
                        .find(|(k, _)| k == "novalue")
                        .map(|(_, v)| v == "yes")
                        .unwrap_or(false);
                    props.push(AddressDataProp { name, novalue });
                }
            }
        }

        Self {
            content_type,
            version,
            allprop,
            props,
        }
    }

    fn to_node(&self) -> Node {
        let mut attrs = Vec::new();
        if self.content_type != "text/vcard" {
            attrs.push(("content-type".into(), self.content_type.clone()));
        }
        if self.version != "3.0" {
            attrs.push(("version".into(), self.version.clone()));
        }

        let children = if self.allprop {
            vec![Node::elem(QName::new(CARDDAV_NS, "allprop"), vec![])]
        } else {
            self.props
                .iter()
                .map(|p| {
                    let mut p_attrs = vec![("name".into(), p.name.clone())];
                    if p.novalue {
                        p_attrs.push(("novalue".into(), "yes".into()));
                    }
                    Node::Elem(Elem {
                        name: QName::new(CARDDAV_NS, "prop"),
                        attributes: p_attrs,
                        children: vec![],
                    })
                })
                .collect()
        };

        Node::Elem(Elem {
            name: QName::new(CARDDAV_NS, "address-data"),
            attributes: attrs,
            children,
        })
    }
}

/// The `C:filter` element (RFC 6352 §10.5).
#[derive(Clone, Debug, PartialEq)]
pub struct Filter {
    pub test: FilterTest,
    pub prop_filters: Vec<PropFilter>,
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            test: FilterTest::AnyOf,
            prop_filters: Vec::new(),
        }
    }
}

/// The `test` attribute of `C:filter` / `C:prop-filter` (RFC 6352 §10.5).
#[derive(Clone, Debug, Default, PartialEq)]
pub enum FilterTest {
    #[default]
    AnyOf,
    AllOf,
}

impl FilterTest {
    fn as_str(&self) -> &'static str {
        match self {
            Self::AnyOf => "anyof",
            Self::AllOf => "allof",
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            "allof" => Self::AllOf,
            _ => Self::AnyOf,
        }
    }
}

/// The `C:prop-filter` element (RFC 6352 §10.5.1).
#[derive(Clone, Debug, PartialEq)]
pub struct PropFilter {
    pub name: String,
    pub test: FilterTest,
    pub content: PropFilterContent,
}

/// The content of a `C:prop-filter`: either `is-not-defined` or a set of
/// `text-match` / `param-filter` children (RFC 6352 §10.5.1).
#[derive(Clone, Debug, PartialEq)]
pub enum PropFilterContent {
    IsNotDefined,
    Match {
        text_matches: Vec<TextMatch>,
        param_filters: Vec<ParamFilter>,
    },
}

/// The `C:param-filter` element (RFC 6352 §10.5.2).
#[derive(Clone, Debug, PartialEq)]
pub struct ParamFilter {
    pub name: String,
    pub content: ParamFilterContent,
}

/// The content of a `C:param-filter` (RFC 6352 §10.5.2).
///
/// An empty `param-filter` (no children) is represented as `Match(None)`,
/// meaning "parameter exists, any value matches". `IsNotDefined` means the
/// parameter must NOT exist.
#[derive(Clone, Debug, PartialEq)]
pub enum ParamFilterContent {
    IsNotDefined,
    Match(Option<TextMatch>),
}

/// The `C:text-match` element (RFC 6352 §10.5.4).
#[derive(Clone, Debug, PartialEq)]
pub struct TextMatch {
    pub collation: String,
    pub negate_condition: bool,
    pub match_type: MatchType,
    pub value: String,
}

impl Default for TextMatch {
    fn default() -> Self {
        Self {
            collation: DEFAULT_COLLATION.into(),
            negate_condition: false,
            match_type: MatchType::Contains,
            value: String::new(),
        }
    }
}

/// The `match-type` attribute of `C:text-match` (RFC 6352 §10.5.4).
#[derive(Clone, Debug, Default, PartialEq)]
pub enum MatchType {
    Equals,
    #[default]
    Contains,
    StartsWith,
    EndsWith,
}

impl MatchType {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Equals => "equals",
            Self::Contains => "contains",
            Self::StartsWith => "starts-with",
            Self::EndsWith => "ends-with",
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            "equals" => Self::Equals,
            "starts-with" => Self::StartsWith,
            "ends-with" => Self::EndsWith,
            _ => Self::Contains,
        }
    }
}

impl TextMatch {
    fn to_node(&self) -> Node {
        let mut attrs = Vec::new();
        if self.collation != DEFAULT_COLLATION {
            attrs.push(("collation".into(), self.collation.clone()));
        }
        if self.negate_condition {
            attrs.push(("negate-condition".into(), "yes".into()));
        }
        if self.match_type != MatchType::Contains {
            attrs.push(("match-type".into(), self.match_type.as_str().into()));
        }
        Node::Elem(Elem {
            name: QName::new(CARDDAV_NS, "text-match"),
            attributes: attrs,
            children: vec![Node::text_node(self.value.clone())],
        })
    }

    fn from_elem(e: &Elem) -> Self {
        let collation = e
            .attributes
            .iter()
            .find(|(k, _)| k == "collation")
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| DEFAULT_COLLATION.into());
        let negate_condition = e
            .attributes
            .iter()
            .find(|(k, _)| k == "negate-condition")
            .map(|(_, v)| v == "yes")
            .unwrap_or(false);
        let match_type = e
            .attributes
            .iter()
            .find(|(k, _)| k == "match-type")
            .map(|(_, v)| MatchType::from_str(v))
            .unwrap_or_default();
        Self {
            collation,
            negate_condition,
            match_type,
            value: e.text(),
        }
    }
}

impl ParamFilter {
    fn to_node(&self) -> Node {
        let children = match &self.content {
            ParamFilterContent::IsNotDefined => {
                vec![Node::elem(QName::new(CARDDAV_NS, "is-not-defined"), vec![])]
            }
            ParamFilterContent::Match(None) => vec![],
            ParamFilterContent::Match(Some(tm)) => vec![tm.to_node()],
        };
        Node::Elem(Elem {
            name: QName::new(CARDDAV_NS, "param-filter"),
            attributes: vec![("name".into(), self.name.clone())],
            children,
        })
    }

    fn from_elem(e: &Elem) -> Self {
        let name = e
            .attributes
            .iter()
            .find(|(k, _)| k == "name")
            .map(|(_, v)| v.clone())
            .unwrap_or_default();
        let content = if e.child_elem(CARDDAV_NS, "is-not-defined").is_some() {
            ParamFilterContent::IsNotDefined
        } else if let Some(tm) = e.child_elem(CARDDAV_NS, "text-match") {
            ParamFilterContent::Match(Some(TextMatch::from_elem(tm)))
        } else {
            ParamFilterContent::Match(None)
        };
        Self { name, content }
    }
}

impl PropFilter {
    fn to_node(&self) -> Node {
        let test_attr = if self.test != FilterTest::AnyOf {
            vec![("test".into(), self.test.as_str().into())]
        } else {
            Vec::new()
        };
        let children = match &self.content {
            PropFilterContent::IsNotDefined => {
                vec![Node::elem(QName::new(CARDDAV_NS, "is-not-defined"), vec![])]
            }
            PropFilterContent::Match {
                text_matches,
                param_filters,
            } => {
                let mut nodes: Vec<Node> = text_matches.iter().map(|tm| tm.to_node()).collect();
                nodes.extend(param_filters.iter().map(|pf| pf.to_node()));
                nodes
            }
        };
        Node::Elem(Elem {
            name: QName::new(CARDDAV_NS, "prop-filter"),
            attributes: {
                let mut attrs = vec![("name".into(), self.name.clone())];
                attrs.extend(test_attr);
                attrs
            },
            children,
        })
    }

    fn from_elem(e: &Elem) -> Self {
        let name = e
            .attributes
            .iter()
            .find(|(k, _)| k == "name")
            .map(|(_, v)| v.clone())
            .unwrap_or_default();
        let test = e
            .attributes
            .iter()
            .find(|(k, _)| k == "test")
            .map(|(_, v)| FilterTest::from_str(v))
            .unwrap_or_default();

        let content = if e.child_elem(CARDDAV_NS, "is-not-defined").is_some() {
            PropFilterContent::IsNotDefined
        } else {
            let text_matches: Vec<TextMatch> = e
                .children
                .iter()
                .filter_map(|c| match c {
                    Node::Elem(ce) if ce.name.is(CARDDAV_NS, "text-match") => {
                        Some(TextMatch::from_elem(ce))
                    }
                    _ => None,
                })
                .collect();
            let param_filters: Vec<ParamFilter> = e
                .children
                .iter()
                .filter_map(|c| match c {
                    Node::Elem(ce) if ce.name.is(CARDDAV_NS, "param-filter") => {
                        Some(ParamFilter::from_elem(ce))
                    }
                    _ => None,
                })
                .collect();
            PropFilterContent::Match {
                text_matches,
                param_filters,
            }
        };

        Self {
            name,
            test,
            content,
        }
    }
}

impl Filter {
    fn to_node(&self) -> Node {
        let test_attr = if self.test != FilterTest::AnyOf {
            vec![("test".into(), self.test.as_str().into())]
        } else {
            Vec::new()
        };
        let children: Vec<Node> = self.prop_filters.iter().map(|pf| pf.to_node()).collect();
        Node::Elem(Elem {
            name: QName::new(CARDDAV_NS, "filter"),
            attributes: test_attr,
            children,
        })
    }

    fn from_elem(e: &Elem) -> Self {
        let test = e
            .attributes
            .iter()
            .find(|(k, _)| k == "test")
            .map(|(_, v)| FilterTest::from_str(v))
            .unwrap_or_default();
        let prop_filters: Vec<PropFilter> = e
            .children
            .iter()
            .filter_map(|c| match c {
                Node::Elem(ce) if ce.name.is(CARDDAV_NS, "prop-filter") => {
                    Some(PropFilter::from_elem(ce))
                }
                _ => None,
            })
            .collect();
        Self { test, prop_filters }
    }
}

/// Build the `D:allprop` / `D:propname` / `D:prop` child nodes for a report
/// body, embedding the `C:address-data` directive inside `D:prop` when present.
fn build_prop_nodes(
    prop: &Option<Propfind>,
    address_data: &Option<AddressDataRequest>,
) -> Vec<Node> {
    match prop {
        Some(Propfind::AllProp) => vec![Node::elem(QName::dav("allprop"), vec![])],
        Some(Propfind::PropName) => vec![Node::elem(QName::dav("propname"), vec![])],
        Some(Propfind::Prop { properties }) => {
            let mut prop_children: Vec<Node> = properties
                .iter()
                .map(|p| Node::elem(p.name.clone(), vec![]))
                .collect();
            if let Some(ad) = address_data {
                prop_children.push(ad.to_node());
            }
            vec![Node::elem(QName::dav("prop"), prop_children)]
        }
        None => vec![],
    }
}

/// Parse `D:prop` children into a `Propfind` and optional `AddressDataRequest`.
fn parse_prop_children(prop_elem: &Elem) -> (Option<Propfind>, Option<AddressDataRequest>) {
    let mut properties = Vec::new();
    let mut address_data = None;
    for pc in &prop_elem.children {
        if let Node::Elem(pe) = pc {
            if pe.name.is(CARDDAV_NS, "address-data") {
                address_data = Some(AddressDataRequest::from_elem(pe));
            } else {
                properties.push(Property::name_only(pe.name.clone()));
            }
        }
    }
    (Some(Propfind::Prop { properties }), address_data)
}

/// A `C:addressbook-query` report request body (RFC 6352 §10.3).
///
/// `<!ELEMENT addressbook-query ((DAV:allprop | DAV:propname | DAV:prop)?, filter, limit?)>`
#[derive(Clone, Debug, PartialEq)]
pub struct AddressbookQuery {
    /// The DAV property request (`allprop`, `propname`, or specific `prop`).
    /// `None` means no prop element (server returns what it chooses).
    pub prop: Option<Propfind>,
    /// The `C:address-data` directive, if present inside `D:prop`.
    pub address_data: Option<AddressDataRequest>,
    pub filter: Filter,
    pub limit: Option<u32>,
}

impl AddressbookQuery {
    pub fn to_xml(&self) -> Vec<u8> {
        let mut children = build_prop_nodes(&self.prop, &self.address_data);
        children.push(self.filter.to_node());

        if let Some(n) = self.limit {
            children.push(Node::elem(
                QName::new(CARDDAV_NS, "limit"),
                vec![Node::elem(
                    QName::new(CARDDAV_NS, "nresults"),
                    vec![Node::text_node(n.to_string())],
                )],
            ));
        }

        serialize_document("addressbook-query", CARDDAV_NS, &children)
    }

    pub fn from_xml(input: &[u8]) -> Result<Self, DavError> {
        let root = parse_document(input)?;
        if !root.name.is(CARDDAV_NS, "addressbook-query") {
            return Err(DavError::XmlParse(format!(
                "expected C:addressbook-query root, got {:?}:{}",
                root.name.namespace, root.name.local
            )));
        }
        Self::from_elem(&root)
    }

    fn from_elem(root: &Elem) -> Result<Self, DavError> {
        let mut prop = None;
        let mut address_data = None;
        let mut filter = Filter::default();
        let mut limit = None;

        for child in &root.children {
            if let Node::Elem(e) = child {
                if e.name.is(DAV_NS, "allprop") {
                    prop = Some(Propfind::AllProp);
                } else if e.name.is(DAV_NS, "propname") {
                    prop = Some(Propfind::PropName);
                } else if e.name.is(DAV_NS, "prop") {
                    let (p, ad) = parse_prop_children(e);
                    prop = p;
                    address_data = ad;
                } else if e.name.is(CARDDAV_NS, "filter") {
                    filter = Filter::from_elem(e);
                } else if e.name.is(CARDDAV_NS, "limit")
                    && let Some(nresults) = e.child_elem(CARDDAV_NS, "nresults")
                {
                    limit = nresults.text().parse().ok();
                }
            }
        }

        Ok(Self {
            prop,
            address_data,
            filter,
            limit,
        })
    }
}

/// A `C:addressbook-multiget` report request body (RFC 6352 §10.7).
///
/// `<!ELEMENT addressbook-multiget ((DAV:allprop | DAV:propname | DAV:prop)?, DAV:href+)>`
#[derive(Clone, Debug, PartialEq)]
pub struct AddressbookMultiget {
    pub prop: Option<Propfind>,
    pub address_data: Option<AddressDataRequest>,
    pub hrefs: Vec<String>,
}

impl AddressbookMultiget {
    pub fn to_xml(&self) -> Vec<u8> {
        let mut children = build_prop_nodes(&self.prop, &self.address_data);

        for href in &self.hrefs {
            children.push(Node::elem(
                QName::dav("href"),
                vec![Node::text_node(href.clone())],
            ));
        }

        serialize_document("addressbook-multiget", CARDDAV_NS, &children)
    }

    pub fn from_xml(input: &[u8]) -> Result<Self, DavError> {
        let root = parse_document(input)?;
        if !root.name.is(CARDDAV_NS, "addressbook-multiget") {
            return Err(DavError::XmlParse(format!(
                "expected C:addressbook-multiget root, got {:?}:{}",
                root.name.namespace, root.name.local
            )));
        }
        Self::from_elem(&root)
    }

    fn from_elem(root: &Elem) -> Result<Self, DavError> {
        let mut prop = None;
        let mut address_data = None;
        let mut hrefs = Vec::new();

        for child in &root.children {
            if let Node::Elem(e) = child {
                if e.name.is(DAV_NS, "allprop") {
                    prop = Some(Propfind::AllProp);
                } else if e.name.is(DAV_NS, "propname") {
                    prop = Some(Propfind::PropName);
                } else if e.name.is(DAV_NS, "prop") {
                    let (p, ad) = parse_prop_children(e);
                    prop = p;
                    address_data = ad;
                } else if e.name.is(DAV_NS, "href") {
                    hrefs.push(e.text());
                }
            }
        }

        Ok(Self {
            prop,
            address_data,
            hrefs,
        })
    }
}

/// A parsed CardDAV report body, for dispatch in `DavHandler::report()`.
#[derive(Clone, Debug)]
pub enum CarddavReport {
    AddressbookQuery(AddressbookQuery),
    AddressbookMultiget(AddressbookMultiget),
}

/// Parse a CardDAV report body and return the typed report.
pub fn parse_report(body: &[u8]) -> Result<CarddavReport, DavError> {
    let root = parse_document(body)?;
    if root.name.is(CARDDAV_NS, "addressbook-query") {
        Ok(CarddavReport::AddressbookQuery(
            AddressbookQuery::from_elem(&root)?,
        ))
    } else if root.name.is(CARDDAV_NS, "addressbook-multiget") {
        Ok(CarddavReport::AddressbookMultiget(
            AddressbookMultiget::from_elem(&root)?,
        ))
    } else {
        Err(DavError::XmlParse(format!(
            "unknown CardDAV report: {:?}:{}",
            root.name.namespace, root.name.local
        )))
    }
}

/// Information about an address book collection, as returned by discovery.
#[derive(Clone, Debug, PartialEq)]
pub struct AddressbookInfo {
    pub href: String,
    pub description: Option<String>,
}
