use crate::parser_internals::ParseContext;
use crate::parser_internals::model::Param;
use crate::parser_internals::property_value::parse_property_single_value;
use crate::property_support::PropertyBase;
use crate::{Field, FieldValue, PropertyValueParseError, Value, v};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeoProperty {
    value: Vec<u8>,
    params: Vec<Param>,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum GeoError {
    #[error("invalid geo format")]
    InvalidFormat,
    #[error("property value parse error")]
    InvalidPropertyValue(#[from] PropertyValueParseError),
}

impl PropertyBase for GeoProperty {
    type Error = GeoError;

    fn name(&self) -> Vec<u8> {
        b"GEO".to_vec()
    }

    fn value(&self) -> Value {
        v!(Field::new(vec![FieldValue::raw(self.value.clone())]))
    }

    fn params(&self) -> &[Param] {
        &self.params
    }
}

impl GeoProperty {
    pub fn parse(value: Vec<u8>, params: Vec<Param>, ctx: ParseContext) -> Result<Self, GeoError> {
        Ok(GeoProperty {
            value: parse_property_single_value(value.as_slice(), ctx.strict)?.into_inner(),
            params,
        })
    }
}
