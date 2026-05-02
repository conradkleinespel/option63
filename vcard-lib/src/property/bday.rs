use crate::parser_internals::ParseContext;
use crate::parser_internals::model::Param;
use crate::parser_internals::property_value::parse_property_single_value;
use crate::property::bday::BdayError::UnsupportedVersion;
use crate::property_support::PropertyBase;
use crate::property_support::date_and_or_time::{
    DateAndOrTimeOrText, DateAndOrTimeOrTextError, parse_date_and_or_time_or_text_for_v40,
    parse_date_or_date_time_for_v30,
};
use crate::{PropertyValueParseError, Value, Version};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BdayProperty {
    value: BdayValue,
    params: Vec<Param>,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum BdayError {
    #[error("invalid bday format")]
    InvalidFormat,
    #[error("invalid bday VALUE parameter")]
    InvalidValueParam,
    #[error("unsupported vCard version for bday")]
    UnsupportedVersion,
    #[error("property value parse error")]
    InvalidPropertyValue(#[from] PropertyValueParseError),
}

impl From<DateAndOrTimeOrTextError> for BdayError {
    fn from(e: DateAndOrTimeOrTextError) -> Self {
        match e {
            DateAndOrTimeOrTextError::InvalidFormat => BdayError::InvalidFormat,
            DateAndOrTimeOrTextError::InvalidValueParam => BdayError::InvalidValueParam,
            DateAndOrTimeOrTextError::UnsupportedVersion => BdayError::UnsupportedVersion,
        }
    }
}

type BdayValue = DateAndOrTimeOrText;

impl PropertyBase for BdayProperty {
    type Error = BdayError;

    fn name(&self) -> Vec<u8> {
        b"BDAY".to_vec()
    }

    fn value(&self) -> Value {
        self.value.to_string().into_bytes().into()
    }

    fn params(&self) -> &[Param] {
        &self.params
    }
}

impl BdayProperty {
    pub fn parse(value: Vec<u8>, params: Vec<Param>, ctx: ParseContext) -> Result<Self, BdayError> {
        let value = parse_property_single_value(value.as_slice(), ctx.strict)?.into_inner();
        match ctx.version {
            Version::V40 => Ok(BdayProperty {
                value: parse_date_and_or_time_or_text_for_v40(value, &params)?,
                params,
            }),
            Version::V30 => Ok(BdayProperty {
                value: parse_date_or_date_time_for_v30(value.as_slice(), &params)
                    .map_err(|_| BdayError::InvalidFormat)?,
                params,
            }),
            // TODO: lax mode should allow this
            _ => Err(UnsupportedVersion),
        }
    }

    pub fn date_and_or_time_or_text(&self) -> &BdayValue {
        &self.value
    }
}
