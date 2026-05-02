pub mod model;
pub mod property_value;
pub mod result;
mod units;
mod utils;
pub mod version;

pub use version::Version;

/// Context for parsing vCard properties
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseContext {
    pub version: Version,
    pub strict: bool,
}

impl ParseContext {
    pub fn new(version: Version, strict: bool) -> Self {
        ParseContext { version, strict }
    }

    pub fn v40_lax() -> Self {
        ParseContext {
            version: Version::V40,
            strict: false,
        }
    }

    pub fn v30_lax() -> Self {
        ParseContext {
            version: Version::V30,
            strict: false,
        }
    }

    pub fn v21_lax() -> Self {
        ParseContext {
            version: Version::V21,
            strict: false,
        }
    }
}
