use crate::vcard::parser::ParseContext;
use crate::vcard::property::param::Param;
use crate::vcard::property::{Field, FieldValue, PropertyBase, Value};
use crate::vcard::property::{PropertyValueParseError, v};
use crate::vcard::property::{parse_property_single_value, run_full};

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
            value: run_full(parse_property_single_value(value.as_slice(), ctx.strict))?
                .into_inner(),
            params,
        })
    }
}
