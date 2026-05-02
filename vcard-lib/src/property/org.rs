use crate::parser_internals::ParseContext;
use crate::parser_internals::model::Param;
use crate::parser_internals::property_value::parse_property_fields_single_value;
use crate::property_support::PropertyBase;
use crate::{FieldValue, PropertyValueParseError, Value, f};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrgProperty {
    value: Vec<FieldValue>,
    params: Vec<Param>,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum OrgError {
    #[error("invalid org format")]
    InvalidFormat,
    #[error("property value parse error")]
    InvalidPropertyValue(#[from] PropertyValueParseError),
}

impl PropertyBase for OrgProperty {
    type Error = OrgError;

    fn name(&self) -> Vec<u8> {
        b"ORG".to_vec()
    }

    fn value(&self) -> Value {
        Value::new(self.value.iter().map(|v| f!(v.clone())).collect())
    }

    fn params(&self) -> &[Param] {
        &self.params
    }
}

impl OrgProperty {
    pub fn parse(value: Vec<u8>, params: Vec<Param>, ctx: ParseContext) -> Result<Self, OrgError> {
        Ok(OrgProperty {
            value: parse_property_fields_single_value(value.as_slice(), ctx.strict)?,
            params,
        })
    }
}
