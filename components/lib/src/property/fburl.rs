use crate::parser_internals::ParseContext;
use crate::parser_internals::model::Param;
use crate::parser_internals::property_value::parse_property_single_value;
use crate::property_support::PropertyBase;
use crate::{PropertyValueParseError, Value};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FburlProperty {
    value: Vec<u8>,
    params: Vec<Param>,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum FburlError {
    #[error("invalid fburl format")]
    InvalidFormat,
    #[error("property value parse error")]
    InvalidPropertyValue(#[from] PropertyValueParseError),
}

impl PropertyBase for FburlProperty {
    type Error = FburlError;

    fn name(&self) -> Vec<u8> {
        b"FBURL".to_vec()
    }

    fn value(&self) -> Value {
        self.value.clone().into()
    }

    fn params(&self) -> &[Param] {
        &self.params
    }
}

impl FburlProperty {
    pub fn parse(
        value: Vec<u8>,
        params: Vec<Param>,
        ctx: ParseContext,
    ) -> Result<Self, FburlError> {
        Ok(FburlProperty {
            value: parse_property_single_value(value.as_slice(), ctx.strict)?.into_inner(),
            params,
        })
    }
}
