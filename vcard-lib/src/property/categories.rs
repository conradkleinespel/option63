use crate::parser_internals::ParseContext;
use crate::parser_internals::model::Param;
use crate::parser_internals::property_value::parse_property_multiple_values;
use crate::property_support::PropertyBase;
use crate::{Field, PropertyValueParseError, Value, fv, v};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CategoriesProperty {
    categories: Vec<Vec<u8>>,
    params: Vec<Param>,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum CategoriesError {
    #[error("invalid categories format")]
    InvalidFormat,
    #[error("property value parse error")]
    InvalidPropertyValue(#[from] PropertyValueParseError),
}

impl PropertyBase for CategoriesProperty {
    type Error = CategoriesError;

    fn name(&self) -> Vec<u8> {
        b"CATEGORIES".to_vec()
    }

    fn value(&self) -> Value {
        v!(Field::new(
            self.categories.iter().map(|c| fv!(c.to_vec())).collect(),
        ))
    }

    fn params(&self) -> &[Param] {
        &self.params
    }
}

impl CategoriesProperty {
    pub fn parse(
        value: Vec<u8>,
        params: Vec<Param>,
        ctx: ParseContext,
    ) -> Result<Self, CategoriesError> {
        Ok(CategoriesProperty {
            categories: parse_property_multiple_values(value.as_slice(), ctx.strict)?
                .into_iter()
                .map(|fv| fv.into_inner())
                .collect(),
            params,
        })
    }
}
