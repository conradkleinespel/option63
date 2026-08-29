use crate::parser_internals::ParseContext;
use crate::parser_internals::model::Param;
use crate::parser_internals::property_value::parse_property_single_value;
use crate::property_support::PropertyBase;
use crate::{PropertyValueParseError, Value};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SoundProperty {
    value: Vec<u8>,
    params: Vec<Param>,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum SoundError {
    #[error("invalid sound format")]
    InvalidFormat,
    #[error("property value parse error")]
    InvalidPropertyValue(#[from] PropertyValueParseError),
}

impl PropertyBase for SoundProperty {
    type Error = SoundError;

    fn name(&self) -> Vec<u8> {
        b"SOUND".to_vec()
    }

    fn value(&self) -> Value {
        self.value.clone().into()
    }

    fn params(&self) -> &[Param] {
        &self.params
    }
}

impl SoundProperty {
    pub fn parse(
        value: Vec<u8>,
        params: Vec<Param>,
        ctx: ParseContext,
    ) -> Result<Self, SoundError> {
        Ok(SoundProperty {
            value: parse_property_single_value(value.as_slice(), ctx.strict)?.into_inner(),
            params,
        })
    }
}
