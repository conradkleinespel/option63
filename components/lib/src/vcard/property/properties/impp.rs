use crate::vcard::parser::ParseContext;
use crate::vcard::property::PropertyValueParseError;
use crate::vcard::property::param::Param;
use crate::vcard::property::{PropertyBase, Value};
use crate::vcard::property::{parse_property_single_value, run_full};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImppProperty {
    value: Vec<u8>,
    params: Vec<Param>,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ImppError {
    #[error("invalid impp format")]
    InvalidFormat,
    #[error("property value parse error")]
    InvalidPropertyValue(#[from] PropertyValueParseError),
}

impl PropertyBase for ImppProperty {
    type Error = ImppError;

    fn name(&self) -> Vec<u8> {
        b"IMPP".to_vec()
    }

    fn value(&self) -> Value {
        self.value.clone().into()
    }

    fn params(&self) -> &[Param] {
        &self.params
    }
}

impl ImppProperty {
    pub fn parse(value: Vec<u8>, params: Vec<Param>, ctx: ParseContext) -> Result<Self, ImppError> {
        Ok(ImppProperty {
            value: run_full(parse_property_single_value(value.as_slice(), ctx.strict))?
                .into_inner(),
            params,
        })
    }
}
