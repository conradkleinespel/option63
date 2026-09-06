use super::BdayError::UnsupportedVersion;
use crate::vcard::Version;
use crate::vcard::parser::ParseContext;
use crate::vcard::property::PropertyValueParseError;
use crate::vcard::property::datetime::DateAndOrTimeOrText;
use crate::vcard::property::param::Param;
use crate::vcard::property::{
    DateAndOrTimeOrTextError, parse_date_and_or_time_or_text_for_v40,
    parse_date_or_date_time_for_v30,
};
use crate::vcard::property::{PropertyBase, Value};
use crate::vcard::property::{parse_property_single_value, run_full};

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
        let value =
            run_full(parse_property_single_value(value.as_slice(), ctx.strict))?.into_inner();
        let (rest, value, params) = match ctx.version {
            Version::V40 => {
                let (rest, value) =
                    parse_date_and_or_time_or_text_for_v40(value.as_slice(), &params).map_err(
                        |err| match err {
                            nom::Err::Error(e) | nom::Err::Failure(e) => e.into(),
                            nom::Err::Incomplete(_) => BdayError::InvalidFormat,
                        },
                    )?;

                (rest, value, params)
            }
            Version::V30 => {
                let (rest, value) = parse_date_or_date_time_for_v30(value.as_slice(), &params)
                    .map_err(|err| match err {
                        nom::Err::Error(e) | nom::Err::Failure(e) => e.into(),
                        nom::Err::Incomplete(_) => BdayError::InvalidFormat,
                    })?;

                (rest, value, params)
            }
            // TODO: lax mode should allow this
            _ => return Err(UnsupportedVersion),
        };

        if !rest.is_empty() {
            Err(BdayError::InvalidFormat)
        } else {
            Ok(BdayProperty { value, params })
        }
    }

    pub fn date_and_or_time_or_text(&self) -> &BdayValue {
        &self.value
    }
}
