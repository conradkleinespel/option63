use crate::parser_internals::ParseContext;
use crate::parser_internals::model::Param;
use crate::parser_internals::property_value::parse_property_fields_and_values;
use crate::property::NError::TooManyFields;
use crate::property_support::PropertyBase;
use crate::{Field, PropertyValueParseError, Value};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NProperty {
    fields: Vec<Field>,
    params: Vec<Param>,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum NError {
    #[error("invalid N format")]
    InvalidFormat,
    #[error("too many fields, expected at most 5, got {0}")]
    TooManyFields(usize),
    #[error("property value parse error")]
    InvalidPropertyValue(#[from] PropertyValueParseError),
}

impl PropertyBase for NProperty {
    type Error = NError;

    fn name(&self) -> Vec<u8> {
        b"N".to_vec()
    }

    fn value(&self) -> Value {
        Value::new(self.fields.clone())
    }

    fn params(&self) -> &[Param] {
        &self.params
    }
}

impl NProperty {
    pub fn parse(value: Vec<u8>, params: Vec<Param>, ctx: ParseContext) -> Result<Self, NError> {
        // Fields represent (each with possibly multiple values):
        // - Family Names (also known as surnames)
        // - Given Names
        // - Additional Names
        // - Honorific Prefixes
        // - Honorific Suffixes
        let fields = parse_property_fields_and_values(value.as_slice(), true, true, ctx.strict)?;

        if fields.len() > 5 {
            return Err(TooManyFields(fields.len()));
        }

        Ok(NProperty { fields, params })
    }
}
