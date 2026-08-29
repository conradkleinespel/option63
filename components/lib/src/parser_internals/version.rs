/// vCard version enumeration.
///
/// This enum is non-exhaustive to allow future versions to be added.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Version {
    /// vCard 2.1 (default if VERSION property is absent)
    V21,
    /// vCard 3.0 (RFC 2426)
    V30,
    /// vCard 4.0 (RFC 6350)
    V40,
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Version::V21 => write!(f, "2.1"),
            Version::V30 => write!(f, "3.0"),
            Version::V40 => write!(f, "4.0"),
        }
    }
}
