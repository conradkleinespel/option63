//! WebDAV Access Control List data models and XML representations according to RFC 3744.
use crate::webdav::error::DavError;
use crate::webdav::property::{Property, PropertyValue};
use crate::webdav::xml::{
    DAV_NS, Elem, FromXmlDocument, FromXmlElem, Node, QName, ToXmlDocument, ToXmlNode,
    parse_document_root, serialize_document,
};

/// A DAV privilege (RFC 3744 §3). The `privilege` element is declared `ANY`,
/// so privileges may be DAV-defined (`read`, `write`, etc) or server-defined
/// in a custom namespace.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Privilege {
    pub name: QName,
}

impl ToXmlNode for Privilege {
    fn to_node(&self) -> Node {
        Node::dav_elem("privilege", vec![Node::elem(self.name.clone(), vec![])])
    }
}

impl FromXmlElem for Privilege {
    fn from_elem(elem: &Elem) -> Result<Self, DavError> {
        // TODO: What does the RFC say about D:privilege with multiple children
        elem.all_child_elems()
            .next()
            .map(|inner| Self {
                name: inner.name.clone(),
            })
            .ok_or_else(|| DavError::XmlParse("D:privilege must contain a child element".into()))
    }
}

impl Privilege {
    pub fn new(name: QName) -> Self {
        Self { name }
    }

    pub fn dav(local: &str) -> Self {
        Self {
            name: QName::dav(local),
        }
    }

    // Well-known DAV privileges (RFC 3744 §3.1–§3.11)
    pub fn read() -> Self {
        Self::dav("read")
    }
    pub fn write() -> Self {
        Self::dav("write")
    }
    pub fn write_properties() -> Self {
        Self::dav("write-properties")
    }
    pub fn write_content() -> Self {
        Self::dav("write-content")
    }
    pub fn unlock() -> Self {
        Self::dav("unlock")
    }
    pub fn read_acl() -> Self {
        Self::dav("read-acl")
    }
    pub fn read_current_user_privilege_set() -> Self {
        Self::dav("read-current-user-privilege-set")
    }
    pub fn write_acl() -> Self {
        Self::dav("write-acl")
    }
    pub fn bind() -> Self {
        Self::dav("bind")
    }
    pub fn unbind() -> Self {
        Self::dav("unbind")
    }
    pub fn all() -> Self {
        Self::dav("all")
    }

    /// Serialize as a `<D:privilege><{name}/></D:privilege>` node.
    pub fn to_node(&self) -> Node {
        <Self as ToXmlNode>::to_node(self)
    }

    /// Parse from a `<D:privilege>` element - its first child element is the
    /// privilege name.
    pub fn from_elem(e: &Elem) -> Result<Self, DavError> {
        <Self as FromXmlElem>::from_elem(e)
    }
}

/// The principal of an ACE (RFC 3744 §5.5.1).
///
/// `<!ELEMENT principal (href | all | authenticated | unauthenticated
///                         | property | self)>`
#[derive(Clone, Debug, PartialEq)]
pub enum AcePrincipal {
    /// `<D:href>` - identifies a specific principal by URL.
    Href(String),
    /// `<D:all/>` - matches every user.
    All,
    /// `<D:authenticated/>` - matches any authenticated user.
    Authenticated,
    /// `<D:unauthenticated/>` - matches any unauthenticated user.
    Unauthenticated,
    /// `<D:property><D:owner/></D:property>` - matches the principal
    /// identified by a property's href value.
    Property(QName),
    /// `<D:self/>` - matches if the resource is a principal and that
    /// principal matches the current user.
    Self_,
}

impl ToXmlNode for AcePrincipal {
    fn to_node(&self) -> Node {
        let children = match self {
            AcePrincipal::Href(url) => vec![Node::href(url.clone())],
            AcePrincipal::All => vec![Node::empty_dav("all")],
            AcePrincipal::Authenticated => vec![Node::empty_dav("authenticated")],
            AcePrincipal::Unauthenticated => vec![Node::empty_dav("unauthenticated")],
            AcePrincipal::Property(name) => {
                vec![Node::dav_elem(
                    "property",
                    vec![Node::elem(name.clone(), vec![])],
                )]
            }
            AcePrincipal::Self_ => vec![Node::empty_dav("self")],
        };
        Node::dav_elem("principal", children)
    }
}

impl FromXmlElem for AcePrincipal {
    fn from_elem(elem: &Elem) -> Result<Self, DavError> {
        for child in elem.all_child_elems() {
            if child.name.is(DAV_NS, "href") {
                return Ok(AcePrincipal::Href(child.text()));
            }
            if child.name.is(DAV_NS, "all") {
                return Ok(AcePrincipal::All);
            }
            if child.name.is(DAV_NS, "authenticated") {
                return Ok(AcePrincipal::Authenticated);
            }
            if child.name.is(DAV_NS, "unauthenticated") {
                return Ok(AcePrincipal::Unauthenticated);
            }
            if child.name.is(DAV_NS, "self") {
                return Ok(AcePrincipal::Self_);
            }
            if child.name.is(DAV_NS, "property") {
                let prop_name = child.all_child_elems().next().map(|c| c.name.clone());
                return match prop_name {
                    Some(name) => Ok(AcePrincipal::Property(name)),
                    None => Err(DavError::XmlParse(
                        "D:property must contain a child element".into(),
                    )),
                };
            }
        }
        Err(DavError::XmlParse(
            "D:principal must contain a child element".into(),
        ))
    }
}

impl AcePrincipal {
    pub fn to_node(&self) -> Node {
        <Self as ToXmlNode>::to_node(self)
    }

    pub fn from_elem(e: &Elem) -> Result<Self, DavError> {
        <Self as FromXmlElem>::from_elem(e)
    }
}

/// Either a principal or an inverted principal (RFC 3744 §5.5.1).
///
/// `<!ELEMENT invert principal>`
#[derive(Clone, Debug, PartialEq)]
pub enum AceSubject {
    Principal(AcePrincipal),
    Invert(AcePrincipal),
}

impl ToXmlNode for AceSubject {
    fn to_node(&self) -> Node {
        match self {
            AceSubject::Principal(p) => p.to_node(),
            AceSubject::Invert(p) => Node::dav_elem("invert", vec![p.to_node()]),
        }
    }
}

impl FromXmlElem for AceSubject {
    fn from_elem(elem: &Elem) -> Result<Self, DavError> {
        if elem.name.is(DAV_NS, "principal") {
            Ok(AceSubject::Principal(AcePrincipal::from_elem(elem)?))
        } else if elem.name.is(DAV_NS, "invert") {
            let inner = elem
                .child_elem(DAV_NS, "principal")
                .ok_or_else(|| DavError::XmlParse("D:invert must contain D:principal".into()))?;
            Ok(AceSubject::Invert(AcePrincipal::from_elem(inner)?))
        } else {
            Err(DavError::XmlParse(format!(
                "expected D:principal or D:invert, got {:?}",
                elem.name.local
            )))
        }
    }
}

impl AceSubject {
    pub fn to_node(&self) -> Node {
        <Self as ToXmlNode>::to_node(self)
    }

    pub fn from_elem(e: &Elem) -> Result<Self, DavError> {
        <Self as FromXmlElem>::from_elem(e)
    }
}

/// Grant or deny a set of privileges (RFC 3744 §5.5.2).
///
/// `<!ELEMENT grant (privilege+)>`
/// `<!ELEMENT deny (privilege+)>`
#[derive(Clone, Debug, PartialEq)]
pub enum GrantOrDeny {
    Grant(Vec<Privilege>),
    Deny(Vec<Privilege>),
}

impl ToXmlNode for GrantOrDeny {
    fn to_node(&self) -> Node {
        let (tag, privileges) = match self {
            GrantOrDeny::Grant(p) => ("grant", p),
            GrantOrDeny::Deny(p) => ("deny", p),
        };
        Node::dav_elem(tag, privileges.iter().map(|p| p.to_node()).collect())
    }
}

impl FromXmlElem for GrantOrDeny {
    fn from_elem(elem: &Elem) -> Result<Self, DavError> {
        let privs: Vec<Privilege> = elem.parse_children(DAV_NS, "privilege")?;
        if privs.is_empty() {
            return Err(DavError::XmlParse(format!(
                "D:{} must contain at least one D:privilege",
                elem.name.local
            )));
        }
        if elem.name.is(DAV_NS, "grant") {
            Ok(GrantOrDeny::Grant(privs))
        } else if elem.name.is(DAV_NS, "deny") {
            Ok(GrantOrDeny::Deny(privs))
        } else {
            Err(DavError::XmlParse(format!(
                "expected D:grant or D:deny, got {:?}",
                elem.name.local
            )))
        }
    }
}

impl GrantOrDeny {
    pub fn to_node(&self) -> Node {
        <Self as ToXmlNode>::to_node(self)
    }

    pub fn from_elem(e: &Elem) -> Result<Self, DavError> {
        <Self as FromXmlElem>::from_elem(e)
    }
}

/// A single ACE (RFC 3744 §5.5).
///
/// `<!ELEMENT ace ((principal | invert), (grant|deny), protected?, inherited?)>`
#[derive(Clone, Debug, PartialEq)]
pub struct Ace {
    pub subject: AceSubject,
    pub grant_or_deny: GrantOrDeny,
    pub protected: bool,
    /// `<D:inherited><D:href>...</D:href></D:inherited>`
    pub inherited: Option<String>,
}

impl ToXmlNode for Ace {
    fn to_node(&self) -> Node {
        let mut children = vec![self.subject.to_node(), self.grant_or_deny.to_node()];
        if self.protected {
            children.push(Node::empty_dav("protected"));
        }
        if let Some(href) = &self.inherited {
            children.push(Node::dav_elem("inherited", vec![Node::href(href.clone())]));
        }
        Node::dav_elem("ace", children)
    }
}

impl FromXmlElem for Ace {
    fn from_elem(elem: &Elem) -> Result<Self, DavError> {
        let mut subject = None;
        let mut grant_or_deny = None;
        let mut protected = false;
        let mut inherited = None;
        for inner in elem.all_child_elems() {
            match inner.name.local.as_str() {
                "principal" | "invert" if inner.name.namespace == DAV_NS => {
                    subject = Some(AceSubject::from_elem(inner)?);
                }
                "grant" | "deny" if inner.name.namespace == DAV_NS => {
                    grant_or_deny = Some(GrantOrDeny::from_elem(inner)?);
                }
                "protected" if inner.name.namespace == DAV_NS => {
                    protected = true;
                }
                "inherited" if inner.name.namespace == DAV_NS => {
                    if let Some(h) = inner.child_elem(DAV_NS, "href") {
                        inherited = Some(h.text());
                    }
                }
                _ => {}
            }
        }
        let subject = subject.ok_or_else(|| {
            DavError::MissingField("D:ace requires D:principal or D:invert".into())
        })?;
        let grant_or_deny = grant_or_deny
            .ok_or_else(|| DavError::MissingField("D:ace requires D:grant or D:deny".into()))?;
        Ok(Self {
            subject,
            grant_or_deny,
            protected,
            inherited,
        })
    }
}

impl Ace {
    pub fn to_node(&self) -> Node {
        <Self as ToXmlNode>::to_node(self)
    }

    pub fn from_elem(e: &Elem) -> Result<Self, DavError> {
        <Self as FromXmlElem>::from_elem(e)
    }
}

/// The `DAV:acl` property (RFC 3744 §5.5).
///
/// `<!ELEMENT acl (ace*)>`
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Acl {
    pub aces: Vec<Ace>,
}

impl ToXmlNode for Acl {
    fn to_node(&self) -> Node {
        Node::dav_elem("acl", self.to_nodes())
    }
}

impl FromXmlElem for Acl {
    fn from_elem(elem: &Elem) -> Result<Self, DavError> {
        let aces = elem.parse_children(DAV_NS, "ace")?;
        Ok(Self { aces })
    }
}

impl ToXmlDocument for Acl {
    fn to_xml(&self) -> Vec<u8> {
        serialize_document("acl", DAV_NS, &self.to_nodes())
    }
}

impl FromXmlDocument for Acl {
    fn from_xml(input: &[u8]) -> Result<Self, DavError> {
        let root = parse_document_root(input, DAV_NS, "acl")?;
        Self::from_elem(&root)
    }
}

impl Acl {
    pub fn to_nodes(&self) -> Vec<Node> {
        self.aces.iter().map(|a| a.to_node()).collect()
    }

    pub fn from_nodes(nodes: &[Node]) -> Result<Self, DavError> {
        let aces = nodes
            .iter()
            .filter_map(|n| match n {
                Node::Elem(e) if e.name.is(DAV_NS, "ace") => Some(Ace::from_elem(e)),
                _ => None,
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { aces })
    }

    pub fn to_xml(&self) -> Vec<u8> {
        <Self as ToXmlDocument>::to_xml(self)
    }

    pub fn from_xml(input: &[u8]) -> Result<Self, DavError> {
        <Self as FromXmlDocument>::from_xml(input)
    }

    /// Construct a `D:acl` property.
    pub fn to_property(&self) -> Property {
        Property::new(QName::dav("acl"), PropertyValue::Raw(self.to_nodes()))
    }
}

/// The `DAV:required-principal` element (RFC 3744 §5.6.4).
///
/// `<!ELEMENT required-principal
///     (all? | authenticated? | unauthenticated? | self? | href* | property*)>`
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RequiredPrincipal {
    pub all: bool,
    pub authenticated: bool,
    pub unauthenticated: bool,
    pub self_principal: bool,
    pub hrefs: Vec<String>,
    pub properties: Vec<QName>,
}

impl ToXmlNode for RequiredPrincipal {
    fn to_node(&self) -> Node {
        let mut children = Vec::new();
        if self.all {
            children.push(Node::empty_dav("all"));
        }
        if self.authenticated {
            children.push(Node::empty_dav("authenticated"));
        }
        if self.unauthenticated {
            children.push(Node::empty_dav("unauthenticated"));
        }
        if self.self_principal {
            children.push(Node::empty_dav("self"));
        }
        for href in &self.hrefs {
            children.push(Node::href(href.clone()));
        }
        for prop in &self.properties {
            children.push(Node::elem(prop.clone(), vec![]));
        }
        Node::dav_elem("required-principal", children)
    }
}

impl FromXmlElem for RequiredPrincipal {
    fn from_elem(elem: &Elem) -> Result<Self, DavError> {
        let mut rp = RequiredPrincipal::default();
        for inner in elem.all_child_elems() {
            match inner.name.local.as_str() {
                "all" if inner.name.namespace == DAV_NS => rp.all = true,
                "authenticated" if inner.name.namespace == DAV_NS => rp.authenticated = true,
                "unauthenticated" if inner.name.namespace == DAV_NS => {
                    rp.unauthenticated = true;
                }
                "self" if inner.name.namespace == DAV_NS => rp.self_principal = true,
                "href" if inner.name.namespace == DAV_NS => rp.hrefs.push(inner.text()),
                "property" if inner.name.namespace == DAV_NS => {
                    // TODO: What does the RFC say about D:property with multiple children
                    if let Some(child) = inner.all_child_elems().next() {
                        rp.properties.push(child.name.clone());
                    }
                }
                _ => rp.properties.push(inner.name.clone()),
            }
        }
        Ok(rp)
    }
}

impl RequiredPrincipal {
    pub fn to_node(&self) -> Node {
        <Self as ToXmlNode>::to_node(self)
    }

    pub fn from_elem(e: &Elem) -> Self {
        <Self as FromXmlElem>::from_elem(e).unwrap_or_default()
    }
}

/// The `DAV:acl-restrictions` property (RFC 3744 §5.6).
///
/// `<!ELEMENT acl-restrictions (grant-only?, no-invert?,
///                              deny-before-grant?, required-principal?)>`
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AclRestrictions {
    pub grant_only: bool,
    pub no_invert: bool,
    pub deny_before_grant: bool,
    pub required_principal: Option<RequiredPrincipal>,
}

impl ToXmlNode for AclRestrictions {
    fn to_node(&self) -> Node {
        Node::dav_elem("acl-restrictions", self.to_nodes())
    }
}

impl FromXmlElem for AclRestrictions {
    fn from_elem(elem: &Elem) -> Result<Self, DavError> {
        Ok(Self::from_nodes(&elem.children))
    }
}

impl AclRestrictions {
    pub fn to_nodes(&self) -> Vec<Node> {
        let mut children = Vec::new();
        if self.grant_only {
            children.push(Node::empty_dav("grant-only"));
        }
        if self.no_invert {
            children.push(Node::empty_dav("no-invert"));
        }
        if self.deny_before_grant {
            children.push(Node::empty_dav("deny-before-grant"));
        }
        if let Some(rp) = &self.required_principal {
            children.push(rp.to_node());
        }
        children
    }

    pub fn from_nodes(nodes: &[Node]) -> Self {
        let mut restrictions = AclRestrictions::default();
        for node in nodes {
            if let Node::Elem(e) = node {
                match e.name.local.as_str() {
                    "grant-only" if e.name.namespace == DAV_NS => restrictions.grant_only = true,
                    "no-invert" if e.name.namespace == DAV_NS => restrictions.no_invert = true,
                    "deny-before-grant" if e.name.namespace == DAV_NS => {
                        restrictions.deny_before_grant = true;
                    }
                    "required-principal" if e.name.namespace == DAV_NS => {
                        restrictions.required_principal = Some(RequiredPrincipal::from_elem(e));
                    }
                    _ => {}
                }
            }
        }
        restrictions
    }

    pub fn to_property(&self) -> Property {
        Property::new(
            QName::dav("acl-restrictions"),
            PropertyValue::Raw(self.to_nodes()),
        )
    }
}

/// A node in the `DAV:supported-privilege-set` tree (RFC 3744 §5.3).
///
/// `<!ELEMENT supported-privilege
///     (privilege, abstract?, description, supported-privilege*)>`
#[derive(Clone, Debug, PartialEq)]
pub struct SupportedPrivilege {
    pub privilege: Privilege,
    pub abstract_: bool,
    pub description: String,
    pub children: Vec<SupportedPrivilege>,
}

impl ToXmlNode for SupportedPrivilege {
    fn to_node(&self) -> Node {
        let mut children = vec![self.privilege.to_node()];
        if self.abstract_ {
            children.push(Node::empty_dav("abstract"));
        }
        children.push(
            Elem::dav("description")
                .with_attr("xml:lang", "en")
                .with_text(self.description.clone())
                .to_node(),
        );
        for child in &self.children {
            children.push(child.to_node());
        }
        Node::dav_elem("supported-privilege", children)
    }
}

impl FromXmlElem for SupportedPrivilege {
    fn from_elem(elem: &Elem) -> Result<Self, DavError> {
        let privilege = elem
            .require_child(DAV_NS, "privilege")
            .map_err(|_| {
                DavError::MissingField("D:supported-privilege requires D:privilege".into())
            })
            .and_then(Privilege::from_elem)?;
        let abstract_ = elem.has_child(DAV_NS, "abstract");
        let description = elem.child_text(DAV_NS, "description").unwrap_or_default();
        let children = elem.parse_children(DAV_NS, "supported-privilege")?;
        Ok(Self {
            privilege,
            abstract_,
            description,
            children,
        })
    }
}

impl SupportedPrivilege {
    pub fn to_node(&self) -> Node {
        <Self as ToXmlNode>::to_node(self)
    }

    pub fn from_elem(e: &Elem) -> Result<Self, DavError> {
        <Self as FromXmlElem>::from_elem(e)
    }
}

/// The `DAV:supported-privilege-set` property (RFC 3744 §5.3).
///
/// `<!ELEMENT supported-privilege-set (supported-privilege*)>`
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SupportedPrivilegeSet {
    pub privileges: Vec<SupportedPrivilege>,
}

impl ToXmlNode for SupportedPrivilegeSet {
    fn to_node(&self) -> Node {
        Node::dav_elem("supported-privilege-set", self.to_nodes())
    }
}

impl FromXmlElem for SupportedPrivilegeSet {
    fn from_elem(elem: &Elem) -> Result<Self, DavError> {
        elem.parse_children(DAV_NS, "supported-privilege")
            .map(|privileges| Self { privileges })
    }
}

impl SupportedPrivilegeSet {
    pub fn to_nodes(&self) -> Vec<Node> {
        self.privileges.iter().map(|p| p.to_node()).collect()
    }

    pub fn from_nodes(nodes: &[Node]) -> Result<Self, DavError> {
        let privileges = nodes
            .iter()
            .filter_map(|n| match n {
                Node::Elem(e) if e.name.is(DAV_NS, "supported-privilege") => {
                    Some(SupportedPrivilege::from_elem(e))
                }
                _ => None,
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { privileges })
    }

    pub fn to_property(&self) -> Property {
        Property::new(
            QName::dav("supported-privilege-set"),
            PropertyValue::Raw(self.to_nodes()),
        )
    }

    /// Build the standard DAV privilege tree per RFC 3744 §5.3.1 / §3.12.
    ///
    /// `all` (abstract, aggregate)
    ///   ├── `read` (aggregate)
    ///   │     ├── `read-acl` (abstract)
    ///   │     └── `read-current-user-privilege-set` (abstract)
    ///   ├── `write` (aggregate, MUST contain bind+unbind+write-properties+write-content)
    ///   │     ├── `write-acl` (abstract)
    ///   │     ├── `write-properties`
    ///   │     ├── `write-content`
    ///   │     ├── `bind`
    ///   │     └── `unbind`
    ///   └── `unlock`
    pub fn standard() -> Self {
        Self {
            privileges: vec![SupportedPrivilege {
                privilege: Privilege::all(),
                abstract_: true,
                description: "All privileges".into(),
                children: vec![
                    SupportedPrivilege {
                        privilege: Privilege::read(),
                        abstract_: false,
                        description: "Read resource".into(),
                        children: vec![
                            SupportedPrivilege {
                                privilege: Privilege::read_acl(),
                                abstract_: true,
                                description: "Read ACL".into(),
                                children: vec![],
                            },
                            SupportedPrivilege {
                                privilege: Privilege::read_current_user_privilege_set(),
                                abstract_: true,
                                description: "Read current user privilege set".into(),
                                children: vec![],
                            },
                        ],
                    },
                    SupportedPrivilege {
                        privilege: Privilege::write(),
                        abstract_: false,
                        description: "Write resource".into(),
                        children: vec![
                            SupportedPrivilege {
                                privilege: Privilege::write_acl(),
                                abstract_: true,
                                description: "Write ACL".into(),
                                children: vec![],
                            },
                            SupportedPrivilege {
                                privilege: Privilege::write_properties(),
                                abstract_: false,
                                description: "Write properties".into(),
                                children: vec![],
                            },
                            SupportedPrivilege {
                                privilege: Privilege::write_content(),
                                abstract_: false,
                                description: "Write content".into(),
                                children: vec![],
                            },
                            SupportedPrivilege {
                                privilege: Privilege::bind(),
                                abstract_: false,
                                description: "Bind".into(),
                                children: vec![],
                            },
                            SupportedPrivilege {
                                privilege: Privilege::unbind(),
                                abstract_: false,
                                description: "Unbind".into(),
                                children: vec![],
                            },
                        ],
                    },
                    SupportedPrivilege {
                        privilege: Privilege::unlock(),
                        abstract_: false,
                        description: "Unlock resource".into(),
                        children: vec![],
                    },
                ],
            }],
        }
    }
}

/// The `DAV:current-user-privilege-set` property (RFC 3744 §5.4).
///
/// `<!ELEMENT current-user-privilege-set (privilege*)>`
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CurrentUserPrivilegeSet {
    pub privileges: Vec<Privilege>,
}

impl ToXmlNode for CurrentUserPrivilegeSet {
    fn to_node(&self) -> Node {
        Node::dav_elem("current-user-privilege-set", self.to_nodes())
    }
}

impl FromXmlElem for CurrentUserPrivilegeSet {
    fn from_elem(elem: &Elem) -> Result<Self, DavError> {
        elem.parse_children(DAV_NS, "privilege")
            .map(|privileges| Self { privileges })
    }
}

impl CurrentUserPrivilegeSet {
    pub fn to_nodes(&self) -> Vec<Node> {
        self.privileges.iter().map(|p| p.to_node()).collect()
    }

    pub fn from_nodes(nodes: &[Node]) -> Result<Self, DavError> {
        let privileges = nodes
            .iter()
            .filter_map(|n| match n {
                Node::Elem(e) if e.name.is(DAV_NS, "privilege") => Some(Privilege::from_elem(e)),
                _ => None,
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { privileges })
    }

    pub fn to_property(&self) -> Property {
        Property::new(
            QName::dav("current-user-privilege-set"),
            PropertyValue::Raw(self.to_nodes()),
        )
    }
}

/// The `DAV:inherited-acl-set` property (RFC 3744 §5.7).
///
/// `<!ELEMENT inherited-acl-set (href*)>`
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InheritedAclSet {
    pub hrefs: Vec<String>,
}

impl ToXmlNode for InheritedAclSet {
    fn to_node(&self) -> Node {
        Node::dav_elem("inherited-acl-set", self.to_nodes())
    }
}

impl FromXmlElem for InheritedAclSet {
    fn from_elem(elem: &Elem) -> Result<Self, DavError> {
        Ok(Self {
            hrefs: elem.child_hrefs(DAV_NS, "href"),
        })
    }
}

impl InheritedAclSet {
    pub fn to_nodes(&self) -> Vec<Node> {
        self.hrefs.iter().map(|h| Node::href(h.clone())).collect()
    }

    pub fn from_nodes(nodes: &[Node]) -> Self {
        let hrefs = nodes
            .iter()
            .filter_map(|n| match n {
                Node::Elem(e) if e.name.is(DAV_NS, "href") => Some(e.text()),
                _ => None,
            })
            .collect();
        Self { hrefs }
    }

    pub fn to_property(&self) -> Property {
        Property::new(
            QName::dav("inherited-acl-set"),
            PropertyValue::Raw(self.to_nodes()),
        )
    }
}

/// A resource that lacked a required privilege (RFC 3744 §7.1.1).
#[derive(Clone, Debug, PartialEq)]
pub struct NeedPrivilegeResource {
    pub href: String,
    pub privilege: Privilege,
}

impl ToXmlNode for NeedPrivilegeResource {
    fn to_node(&self) -> Node {
        Node::dav_elem(
            "resource",
            vec![Node::href(self.href.clone()), self.privilege.to_node()],
        )
    }
}

impl FromXmlElem for NeedPrivilegeResource {
    fn from_elem(elem: &Elem) -> Result<Self, DavError> {
        let href = elem.require_child_text(DAV_NS, "href")?;
        let privilege = elem.require_parsed_child(DAV_NS, "privilege")?;
        Ok(Self { href, privilege })
    }
}

/// The `DAV:need-privileges` error element (RFC 3744 §7.1.1).
///
/// `<!ELEMENT need-privileges (resource)*>`
/// `<!ELEMENT resource (href, privilege)>`
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NeedPrivileges {
    pub resources: Vec<NeedPrivilegeResource>,
}

impl ToXmlNode for NeedPrivileges {
    fn to_node(&self) -> Node {
        Node::dav_elem("need-privileges", self.to_nodes())
    }
}

impl FromXmlElem for NeedPrivileges {
    fn from_elem(elem: &Elem) -> Result<Self, DavError> {
        let resources = elem.parse_children(DAV_NS, "resource")?;
        Ok(Self { resources })
    }
}

impl NeedPrivileges {
    pub fn to_nodes(&self) -> Vec<Node> {
        self.resources.iter().map(|r| r.to_node()).collect()
    }

    /// Wrap in a `D:error` document body (for 403 responses).
    pub fn to_error_xml(&self) -> Vec<u8> {
        let children = vec![self.to_node()];
        serialize_document("error", DAV_NS, &children)
    }
}

/// Construct a `D:resourcetype` property that includes the `D:principal`
/// element, indicating a principal resource (RFC 3744 §4).
pub fn principal_resourcetype() -> Property {
    Property::new(
        QName::dav("resourcetype"),
        PropertyValue::Raw(vec![Node::elem(QName::dav("principal"), vec![])]),
    )
}
