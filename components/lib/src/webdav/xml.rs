//! XML document parsing, node tree representation, and serialization helpers for WebDAV protocols.
use crate::webdav::error::DavError;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::name::ResolveResult;
use quick_xml::{NsReader, Writer};
use std::io::Cursor;

pub const DAV_NS: &str = "DAV:";

/// Trait for types that can be converted into an XML [`Node`].
pub trait ToXmlNode {
    fn to_node(&self) -> Node;
}

/// Trait for types that can be parsed from an XML [`Elem`].
pub trait FromXmlElem: Sized {
    fn from_elem(elem: &Elem) -> Result<Self, DavError>;
}

/// Trait for document-level WebDAV requests or responses that serialize to a full XML document.
pub trait ToXmlDocument {
    fn to_xml(&self) -> Vec<u8>;
}

/// Trait for document-level WebDAV requests or responses that deserialize from XML document bytes.
pub trait FromXmlDocument: Sized {
    fn from_xml(input: &[u8]) -> Result<Self, DavError>;
}

/// A qualified XML name: a local name plus its resolved namespace.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct QName {
    pub namespace: String,
    pub local: String,
}

impl QName {
    pub fn new(namespace: impl Into<String>, local: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            local: local.into(),
        }
    }

    pub fn dav(local: impl Into<String>) -> Self {
        Self::new(DAV_NS, local)
    }

    pub fn is(&self, namespace: &str, local: &str) -> bool {
        self.namespace == namespace && self.local == local
    }
}

/// A parsed XML element with resolved namespaces and attributes.
#[derive(Clone, Debug, PartialEq)]
pub struct Elem {
    pub name: QName,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<Node>,
}

impl Elem {
    /// Merge consecutive `Node::Text` nodes into a single `Node::Text`.
    /// The XML parser may emit separate text events at line boundaries;
    /// merging them ensures `PropertyValue::from_nodes` can classify the
    /// result as `Text` rather than falling back to `Raw`.
    fn with_merged_text_nodes(mut self) -> Elem {
        let mut merged = Vec::with_capacity(self.children.len());
        let mut buf = String::new();
        let mut has_text = false;
        for child in self.children {
            match child {
                Node::Text(t) => {
                    has_text = true;
                    buf.push_str(&t);
                }
                other => {
                    if has_text {
                        merged.push(Node::Text(std::mem::take(&mut buf)));
                        has_text = false;
                    }
                    merged.push(other);
                }
            }
        }
        if has_text {
            merged.push(Node::Text(buf));
        }
        self.children = merged;
        self
    }
}

/// A child of an XML element: either a nested [`Elem`] or text content.
#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    Elem(Elem),
    Text(String),
}

impl Node {
    pub fn elem(qname: QName, children: Vec<Node>) -> Self {
        Node::Elem(Elem {
            name: qname,
            attributes: Vec::new(),
            children,
        })
    }

    pub fn dav_elem(local: impl Into<String>, children: Vec<Node>) -> Self {
        Self::elem(QName::dav(local), children)
    }

    pub fn text_node(value: impl Into<String>) -> Self {
        Node::Text(value.into())
    }

    pub fn empty(qname: QName) -> Self {
        Node::elem(qname, Vec::new())
    }

    pub fn empty_dav(local: impl Into<String>) -> Self {
        Self::dav_elem(local, Vec::new())
    }

    pub fn href(url: impl Into<String>) -> Self {
        Node::dav_elem("href", vec![Node::text_node(url)])
    }

    /// Direct child elements matching the given namespace and local name.
    pub fn child_elems<'a>(
        &'a self,
        namespace: &'a str,
        local: &'a str,
    ) -> impl Iterator<Item = &'a Elem> + 'a {
        self.elems().filter(move |e| e.name.is(namespace, local))
    }

    fn elems(&self) -> impl Iterator<Item = &Elem> {
        match self {
            Node::Elem(e) => e
                .children
                .iter()
                .filter_map(|c| match c {
                    Node::Elem(e) => Some(e),
                    Node::Text(_) => None,
                })
                .collect::<Vec<_>>()
                .into_iter(),
            Node::Text(_) => Vec::new().into_iter(),
        }
    }

    /// Concatenated text of all direct text children.
    pub fn text(&self) -> String {
        match self {
            Node::Elem(e) => e.text(),
            Node::Text(t) => t.clone(),
        }
    }
}

impl Elem {
    pub fn new(name: QName) -> Self {
        Self {
            name,
            attributes: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn dav(local: impl Into<String>) -> Self {
        Self::new(QName::dav(local))
    }

    pub fn with_child(mut self, node: Node) -> Self {
        self.children.push(node);
        self
    }

    pub fn with_children(mut self, nodes: impl IntoIterator<Item = Node>) -> Self {
        self.children.extend(nodes);
        self
    }

    pub fn with_attr(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.push((name.into(), value.into()));
        self
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.children.push(Node::Text(text.into()));
        self
    }

    pub fn to_node(self) -> Node {
        Node::Elem(self)
    }

    /// Direct child elements matching the given namespace and local name.
    pub fn child_elems<'a>(
        &'a self,
        namespace: &'a str,
        local: &'a str,
    ) -> impl Iterator<Item = &'a Elem> + 'a {
        self.children.iter().filter_map(move |c| match c {
            Node::Elem(e) if e.name.is(namespace, local) => Some(e),
            _ => None,
        })
    }

    /// All direct child elements.
    pub fn all_child_elems(&self) -> impl Iterator<Item = &Elem> {
        self.children.iter().filter_map(|c| match c {
            Node::Elem(e) => Some(e),
            _ => None,
        })
    }

    /// Find the first direct child element by (namespace, local).
    pub fn child_elem(&self, namespace: &str, local: &str) -> Option<&Elem> {
        self.children.iter().find_map(|c| match c {
            Node::Elem(e) if e.name.is(namespace, local) => Some(e),
            _ => None,
        })
    }

    /// Require a direct child element, returning an error if missing.
    pub fn require_child(&self, namespace: &str, local: &str) -> Result<&Elem, DavError> {
        self.child_elem(namespace, local).ok_or_else(|| {
            DavError::XmlParse(format!(
                "missing required element <{local}> in <{}>",
                self.name.local
            ))
        })
    }

    /// Concatenated text content of this element's direct text children.
    pub fn text(&self) -> String {
        self.children
            .iter()
            .filter_map(|c| match c {
                Node::Text(t) => Some(t.clone()),
                Node::Elem(_) => None,
            })
            .collect()
    }

    /// Text of the first matching direct child element, if found.
    pub fn child_text(&self, namespace: &str, local: &str) -> Option<String> {
        self.child_elem(namespace, local).map(|e| e.text())
    }

    /// Require text of a direct child element, returning an error if element is missing.
    pub fn require_child_text(&self, namespace: &str, local: &str) -> Result<String, DavError> {
        self.require_child(namespace, local).map(|e| e.text())
    }

    /// Whether this element has a direct child element matching (namespace, local).
    pub fn has_child(&self, namespace: &str, local: &str) -> bool {
        self.child_elem(namespace, local).is_some()
    }

    /// Extract text from a `<D:href>` inside the child element `<namespace:local>`,
    /// or if `<namespace:local>` is itself `<D:href>`, its own text.
    pub fn child_href(&self, namespace: &str, local: &str) -> Option<String> {
        self.child_elem(namespace, local).and_then(|e| {
            if e.name.is(DAV_NS, "href") {
                Some(e.text())
            } else {
                e.child_text(DAV_NS, "href")
            }
        })
    }

    /// Extract all `<D:href>` text values inside the child element `<namespace:local>`,
    /// or all matching `<namespace:local>` children's text if `namespace == DAV_NS && local == "href"`.
    pub fn child_hrefs(&self, namespace: &str, local: &str) -> Vec<String> {
        if namespace == DAV_NS && local == "href" {
            self.children
                .iter()
                .filter_map(|c| match c {
                    Node::Elem(e) if e.name.is(DAV_NS, "href") => Some(e.text()),
                    _ => None,
                })
                .collect()
        } else if let Some(parent) = self.child_elem(namespace, local) {
            parent
                .children
                .iter()
                .filter_map(|c| match c {
                    Node::Elem(e) if e.name.is(DAV_NS, "href") => Some(e.text()),
                    _ => None,
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Parse all child elements matching (namespace, local) into `Vec<T>`.
    pub fn parse_children<T: FromXmlElem>(
        &self,
        namespace: &str,
        local: &str,
    ) -> Result<Vec<T>, DavError> {
        self.child_elems(namespace, local)
            .map(T::from_elem)
            .collect()
    }

    /// Parse an optional child element matching (namespace, local).
    pub fn parse_child<T: FromXmlElem>(
        &self,
        namespace: &str,
        local: &str,
    ) -> Result<Option<T>, DavError> {
        self.child_elem(namespace, local)
            .map(T::from_elem)
            .transpose()
    }

    /// Parse a required child element matching (namespace, local).
    pub fn require_parsed_child<T: FromXmlElem>(
        &self,
        namespace: &str,
        local: &str,
    ) -> Result<T, DavError> {
        let elem = self.require_child(namespace, local)?;
        T::from_elem(elem)
    }
}

/// Parse an XML document and verify that the root matches the expected namespace and local name,
/// returning the root `Elem`.
pub fn parse_document_root(
    input: &[u8],
    expected_namespace: &str,
    expected_local: &str,
) -> Result<Elem, DavError> {
    let root = parse_document(input)?;
    if !root.name.is(expected_namespace, expected_local) {
        return Err(DavError::XmlParse(format!(
            "expected <{expected_local}> root in namespace '{expected_namespace}', got <{}> in namespace '{}'",
            root.name.local, root.name.namespace
        )));
    }
    Ok(root)
}

/// Parse a full XML document into a namespace-resolved tree.
///
/// Namespace declarations (`xmlns` / `xmlns:prefix`) are honored in scope as
/// they are encountered, so `QName.namespace` holds the resolved URI.
pub fn parse_document(input: &[u8]) -> Result<Elem, DavError> {
    let text = std::str::from_utf8(input)
        .map_err(|e| DavError::XmlParse(format!("document not UTF-8: {e}")))?;
    let mut reader = NsReader::from_str(text);
    let mut stack: Vec<Elem> = Vec::new();

    loop {
        match reader.read_resolved_event() {
            Ok((ns, Event::Start(e))) => {
                stack.push(build_elem(ns, &e)?);
            }
            Ok((ns, Event::Empty(e))) => {
                let node = Node::Elem(build_elem(ns, &e)?);
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(node);
                } else if stack.is_empty() {
                    return Ok(match node {
                        Node::Elem(e) => e.with_merged_text_nodes(),
                        _ => unreachable!("empty is always an element"),
                    });
                }
            }
            Ok((_, Event::End(_))) => {
                let elem = stack
                    .pop()
                    .ok_or_else(|| DavError::XmlParse("unmatched closing tag".into()))?
                    .with_merged_text_nodes();
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(Node::Elem(elem));
                } else {
                    return Ok(elem);
                }
            }
            Ok((_, Event::Text(t))) => {
                let text = t.into_inner().into_owned();
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(Node::Text(text));
                } else if !text.trim().is_empty() {
                    return Err(DavError::XmlParse("text outside element boundaries".into()));
                }
            }
            Ok((_, Event::CData(t))) => {
                let text = t.into_inner().into_owned();
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(Node::Text(text));
                } else if !text.trim().is_empty() {
                    return Err(DavError::XmlParse("text outside element boundaries".into()));
                }
            }
            Ok((_, Event::Eof)) => {
                if stack.is_empty() {
                    return Err(DavError::XmlParse("empty document".into()));
                }
                return Err(DavError::XmlParse("unclosed tag".into()));
            }
            // XML comments (`<!-- ... -->`) are ignored as they do not affect document semantics.
            Ok((_, Event::Comment(_))) => {}
            // XML declarations (`<?xml ... ?>`) are ignored because encoding/version is handled at the transport layer.
            Ok((_, Event::Decl(_))) => {}
            // Processing instructions (`<?target ... ?>`) are ignored as WebDAV/CardDAV XML payloads do not use them.
            Ok((_, Event::PI(_))) => {}
            // Document type declarations (`<!DOCTYPE ... >`) are ignored as DTDs are not processed.
            Ok((_, Event::DocType(_))) => {}
            // General entity references (e.g. `&quot;`, `&amp;`) - resolve standard predefined XML entities.
            Ok((_, Event::GeneralRef(r))) => {
                let entity_name = r.as_ref();
                let resolved = match entity_name {
                    "quot" => Some("\""),
                    "amp" => Some("&"),
                    "lt" => Some("<"),
                    "gt" => Some(">"),
                    "apos" => Some("'"),
                    _ => None,
                };
                if let Some(text) = resolved {
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(Node::Text(text.to_string()));
                    } else {
                        return Err(DavError::XmlParse("text outside element boundaries".into()));
                    }
                }
            }
            Err(e) => return Err(DavError::XmlParse(e.to_string())),
        }
    }
}

fn build_elem(ns: ResolveResult, e: &BytesStart) -> Result<Elem, DavError> {
    let namespace = match ns {
        ResolveResult::Bound(n) => n.as_ref().to_string(),
        ResolveResult::Unbound | ResolveResult::Unknown(_) => String::new(),
    };
    let local = e.local_name().as_ref().to_string();

    let mut attributes = Vec::new();
    for attr in e.attributes() {
        let attr = attr.map_err(|err| DavError::XmlParse(err.to_string()))?;
        let k = attr.key.as_ref().to_string();
        let v = attr.value.into_owned();
        // skip xmlns declarations; they are handled by scope tracking
        if k == "xmlns" || k.starts_with("xmlns:") {
            continue;
        }
        attributes.push((k, v));
    }
    Ok(Elem {
        name: QName::new(namespace, local),
        attributes,
        children: Vec::new(),
    })
}

/// Serialize a full document: declaration + a root element in the given
/// namespace. The root and all `DAV:` namespace elements are written with the
/// `D:` prefix; any other namespace (e.g. `carddav`) is written with its
/// default `ns` prefix declared on the root.
pub fn serialize_document(root_local: &str, root_ns: &str, children: &[Node]) -> Vec<u8> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer
        .write_event(Event::Decl(quick_xml::events::BytesDecl::new(
            "1.0",
            Some("utf-8"),
            None,
        )))
        .ok();

    let (root_prefix, root_local_out) = if root_ns == DAV_NS {
        ("D".to_string(), root_local.to_string())
    } else {
        ("ns".to_string(), root_local.to_string())
    };

    let mut root = BytesStart::new(format!("{root_prefix}:{root_local_out}"));
    root.push_attribute(("xmlns:D", DAV_NS));
    if root_ns != DAV_NS {
        root.push_attribute(("xmlns:ns", root_ns));
    } else {
        if let Some(ns) = first_non_dav_namespace(children) {
            root.push_attribute(("xmlns:ns", ns.as_str()));
        }
    }
    writer.write_event(Event::Start(root.clone())).ok();
    write_node_body(&mut writer, children);
    writer
        .write_event(Event::End(BytesEnd::new(format!(
            "{root_prefix}:{root_local_out}"
        ))))
        .ok();
    writer.into_inner().into_inner()
}

/// Find the first non-DAV namespace used in the tree, if any.
fn first_non_dav_namespace(nodes: &[Node]) -> Option<String> {
    for node in nodes {
        match node {
            Node::Elem(e) => {
                if e.name.namespace != DAV_NS && !e.name.namespace.is_empty() {
                    return Some(e.name.namespace.clone());
                }
                if let Some(ns) = first_non_dav_namespace(&e.children) {
                    return Some(ns);
                }
            }
            Node::Text(_) => {}
        }
    }
    None
}

fn write_node(writer: &mut Writer<Cursor<Vec<u8>>>, node: &Node) {
    match node {
        Node::Text(t) => {
            writer
                .write_event(Event::Text(BytesText::new(t.as_str())))
                .ok();
        }
        Node::Elem(elem) => {
            let (prefix, local) = prefix_for(&elem.name);
            let mut start = BytesStart::new(format!("{prefix}{local}"));
            for (k, v) in &elem.attributes {
                start.push_attribute((k.as_str(), v.as_str()));
            }
            writer.write_event(Event::Start(start)).ok();
            for child in &elem.children {
                write_node(writer, child);
            }
            writer
                .write_event(Event::End(BytesEnd::new(format!("{prefix}{local}"))))
                .ok();
        }
    }
}

/// Write the direct children of the root without wrapping them (used so the
/// root's start/end tags written by the caller are respected).
fn write_node_body(writer: &mut Writer<Cursor<Vec<u8>>>, nodes: &[Node]) {
    for node in nodes {
        write_node(writer, node);
    }
}

/// Serialize a list of nodes to XML fragment bytes (no declaration/wrapper).
pub fn serialize_nodes(nodes: &[Node]) -> Vec<u8> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    write_node_body(&mut writer, nodes);
    writer.into_inner().into_inner()
}

/// Return the (prefix, local) tag name for a qualified name. `DAV:` uses the
/// `D` prefix; everything else uses `ns`.
fn prefix_for(name: &QName) -> (String, String) {
    if name.namespace == DAV_NS {
        ("D:".to_string(), name.local.clone())
    } else {
        ("ns:".to_string(), name.local.clone())
    }
}
