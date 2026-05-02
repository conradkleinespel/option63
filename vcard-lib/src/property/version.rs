use crate::Value;
use crate::Version;
use crate::parser_internals::ParseContext;
use crate::parser_internals::model::Param;
use crate::property_support::PropertyBase;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VersionProperty {
    version: Version,
    params: Vec<Param>,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum VersionError {
    #[error("invalid version value, must be 2.1, 3.0, or 4.0")]
    InvalidVersion,
}

impl PropertyBase for VersionProperty {
    type Error = VersionError;

    fn name(&self) -> Vec<u8> {
        b"VERSION".to_vec()
    }

    fn value(&self) -> Value {
        match self.version {
            Version::V21 => b"2.1".to_vec().into(),
            Version::V30 => b"3.0".to_vec().into(),
            Version::V40 => b"4.0".to_vec().into(),
        }
    }

    fn params(&self) -> &[Param] {
        &self.params
    }
}

impl VersionProperty {
    pub fn parse(
        value: Vec<u8>,
        params: Vec<Param>,
        _ctx: ParseContext,
    ) -> Result<Self, VersionError> {
        let version = match value.as_slice() {
            b"2.1" => Version::V21,
            b"3.0" => Version::V30,
            b"4.0" => Version::V40,
            _ => return Err(VersionError::InvalidVersion),
        };

        Ok(VersionProperty { version, params })
    }

    pub fn version(&self) -> Version {
        self.version
    }
}
