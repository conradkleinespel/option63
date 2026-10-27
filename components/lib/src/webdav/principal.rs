//! WebDAV principal property constructors and helpers for user and group resources according to RFC 5397 and RFC 3744.

use crate::webdav::property::{Property, PropertyValue};
use crate::webdav::xml::QName;

/// Construct a `D:principal-URL` property (RFC 3744 §4.2).
///
/// `<!ELEMENT principal-URL (href)>`
pub fn principal_url_property(url: &str) -> Property {
    Property::new(QName::dav("principal-URL"), PropertyValue::Href(url.into()))
}

/// Construct a `D:alternate-URI-set` property (RFC 3744 §4.1).
///
/// `<!ELEMENT alternate-URI-set (href*)>`
pub fn alternate_uri_set_property(urls: &[String]) -> Property {
    Property::new(
        QName::dav("alternate-URI-set"),
        PropertyValue::Hrefs(urls.to_vec()),
    )
}

/// Construct a `D:group-member-set` property (RFC 3744 §4.3).
///
/// `<!ELEMENT group-member-set (href*)>`
pub fn group_member_set_property(urls: &[String]) -> Property {
    Property::new(
        QName::dav("group-member-set"),
        PropertyValue::Hrefs(urls.to_vec()),
    )
}

/// Construct a `D:group-membership` property (RFC 3744 §4.4).
///
/// `<!ELEMENT group-membership (href*)>`
pub fn group_membership_property(urls: &[String]) -> Property {
    Property::new(
        QName::dav("group-membership"),
        PropertyValue::Hrefs(urls.to_vec()),
    )
}

/// Construct a `D:principal-collection-set` property (RFC 3744 §5.8).
///
/// `<!ELEMENT principal-collection-set (href*)>`
pub fn principal_collection_set_property(urls: &[String]) -> Property {
    Property::new(
        QName::dav("principal-collection-set"),
        PropertyValue::Hrefs(urls.to_vec()),
    )
}

/// Construct a `D:owner` property (RFC 3744 §5.1).
///
/// `<!ELEMENT owner (href?)>`
pub fn owner_property(url: Option<&str>) -> Property {
    match url {
        Some(u) => Property::new(QName::dav("owner"), PropertyValue::Href(u.into())),
        None => Property::new(QName::dav("owner"), PropertyValue::Empty),
    }
}

/// Construct a `D:group` property (RFC 3744 §5.2).
///
/// `<!ELEMENT group (href?)>`
pub fn group_property(url: Option<&str>) -> Property {
    match url {
        Some(u) => Property::new(QName::dav("group"), PropertyValue::Href(u.into())),
        None => Property::new(QName::dav("group"), PropertyValue::Empty),
    }
}
