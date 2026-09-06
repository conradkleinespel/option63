use crate::vcard::Version;
use crate::vcard::parser::ParseContext;
use crate::vcard::property::PropertyBase;
use crate::vcard::property::Value;
use crate::vcard::property::param::Param;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VersionProperty {
    version: Version,
    params: Vec<Param>,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum VersionError {
    #[error("unsupported version '{0}', must be 2.1, 3.0, or 4.0")]
    UnsupportedVersion(String),
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
            v => {
                return Err(VersionError::UnsupportedVersion(
                    str::from_utf8(v)
                        .map_err(|_| VersionError::InvalidVersion)?
                        .to_string(),
                ));
            }
        };

        Ok(VersionProperty { version, params })
    }

    pub fn version(&self) -> Version {
        self.version
    }
}
