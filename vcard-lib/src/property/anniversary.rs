use crate::parser_internals::ParseContext;
use crate::parser_internals::model::Param;
use crate::parser_internals::property_value::parse_property_single_value;
use crate::property::anniversary::AnniversaryError::UnsupportedVersion;
use crate::property_support::PropertyBase;
use crate::property_support::date_and_or_time::{
    DateAndOrTimeOrText, DateAndOrTimeOrTextError, parse_date_and_or_time_or_text_for_v40,
};
use crate::{PropertyValueParseError, Value, Version};

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
        let value = parse_property_single_value(value.as_slice(), ctx.strict)?.into_inner();
        match ctx.version {
            Version::V40 => Ok(AnniversaryProperty {
                value: parse_date_and_or_time_or_text_for_v40(value, &params)?,
                params,
            }),
            // TODO: lax mode should allow this
            _ => Err(UnsupportedVersion),
        }
    }

    pub fn date_and_or_time_or_text(&self) -> &AnniversaryValue {
        &self.value
    }
}
