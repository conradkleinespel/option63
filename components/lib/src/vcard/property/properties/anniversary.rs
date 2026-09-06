use super::AnniversaryError::UnsupportedVersion;
use crate::vcard::Version;
use crate::vcard::parser::ParseContext;
use crate::vcard::property::PropertyValueParseError;
use crate::vcard::property::datetime::DateAndOrTimeOrText;
use crate::vcard::property::param::Param;
use crate::vcard::property::{DateAndOrTimeOrTextError, parse_date_and_or_time_or_text_for_v40};
use crate::vcard::property::{PropertyBase, Value};
use crate::vcard::property::{parse_property_single_value, run_full};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnniversaryProperty {
    value: AnniversaryValue,
    params: Vec<Param>,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum AnniversaryError {
    #[error("invalid anniversary format")]
    InvalidFormat,
    #[error("invalid anniversary VALUE parameter")]
    InvalidValueParam,
    #[error("unsupported vCard version for anniversary")]
    UnsupportedVersion,
    #[error("property value parse error")]
    InvalidPropertyValue(#[from] PropertyValueParseError),
}

impl From<DateAndOrTimeOrTextError> for AnniversaryError {
    fn from(e: DateAndOrTimeOrTextError) -> Self {
        match e {
            DateAndOrTimeOrTextError::InvalidFormat => AnniversaryError::InvalidFormat,
            DateAndOrTimeOrTextError::InvalidValueParam => AnniversaryError::InvalidValueParam,
            DateAndOrTimeOrTextError::UnsupportedVersion => AnniversaryError::UnsupportedVersion,
        }
    }
}

type AnniversaryValue = DateAndOrTimeOrText;

impl PropertyBase for AnniversaryProperty {
    type Error = AnniversaryError;

    fn name(&self) -> Vec<u8> {
        b"ANNIVERSARY".to_vec()
    }

    fn value(&self) -> Value {
        self.value.to_string().into_bytes().into()
    }

    fn params(&self) -> &[Param] {
        &self.params
    }
}

impl AnniversaryProperty {
    pub fn parse(
        value: Vec<u8>,
        params: Vec<Param>,
        ctx: ParseContext,
    ) -> Result<Self, AnniversaryError> {
        let value =
            run_full(parse_property_single_value(value.as_slice(), ctx.strict))?.into_inner();
        match ctx.version {
            Version::V40 => {
                let (rest, value) =
                    parse_date_and_or_time_or_text_for_v40(value.as_slice(), &params).map_err(
                        |err| match err {
                            nom::Err::Error(e) | nom::Err::Failure(e) => e.into(),
                            nom::Err::Incomplete(_) => AnniversaryError::InvalidFormat,
                        },
                    )?;

                if !rest.is_empty() {
                    Err(AnniversaryError::InvalidFormat)
                } else {
                    Ok(AnniversaryProperty { value, params })
                }
            }
            // TODO: lax mode should allow this
            _ => Err(UnsupportedVersion),
        }
    }

    pub fn date_and_or_time_or_text(&self) -> &AnniversaryValue {
        &self.value
    }
}
