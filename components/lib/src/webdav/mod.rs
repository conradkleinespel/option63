//! WebDAV module root exporting protocol types, XML helpers, error models, and client and server interfaces.

pub mod acl;
pub mod acl_reports;
pub mod actix_server;
pub mod auth;
pub mod client;
pub mod error;
pub mod multistatus;
pub mod principal;
pub mod property;
pub mod reports;
pub mod requests;
pub mod resource;
pub mod server;
pub mod status;
pub mod sync;
pub mod xml;

pub use crate::webdav::acl::{
    Ace, AcePrincipal, AceSubject, Acl, AclRestrictions, CurrentUserPrivilegeSet, GrantOrDeny,
    InheritedAclSet, NeedPrivilegeResource, NeedPrivileges, Privilege, RequiredPrincipal,
    SupportedPrivilege, SupportedPrivilegeSet, principal_resourcetype,
};
pub use crate::webdav::acl_reports::{
    AclPrincipalPropSet, PrincipalMatch, PrincipalMatchType, PrincipalPropertySearch,
    PrincipalSearchProperty, PrincipalSearchPropertySet, PropertySearch,
};
pub use crate::webdav::multistatus::{
    Multistatus, PropResponse, PropResponseKind, Propstat, Status,
};
pub use crate::webdav::principal::{
    alternate_uri_set_property, group_member_set_property, group_membership_property,
    group_property, owner_property, principal_collection_set_property, principal_url_property,
};
pub use crate::webdav::property::{Property, PropertyValue};
pub use crate::webdav::reports::{ExpandProperty, ExpandPropertyEntry, SupportedReportSet};
pub use crate::webdav::requests::{Propfind, Proppatch};
pub use crate::webdav::status::{Depth, StatusCode};
pub use crate::webdav::sync::{SyncCollection, SyncLevel, sync_token_property};
pub use crate::webdav::xml::{
    DAV_NS, Elem, FromXmlDocument, FromXmlElem, Node, QName, ToXmlDocument, ToXmlNode,
    parse_document, parse_document_root, serialize_document, serialize_nodes,
};
pub use error::DavError;
pub use error::DavErrorBody;
pub use error::dav_error;
