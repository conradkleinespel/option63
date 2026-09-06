use crate::vcard::parser::ParseContext;
use crate::vcard::property::PropertyValueParseError;
use crate::vcard::property::param::Param;
use crate::vcard::property::{Field, PropertyBase, Value};
use crate::vcard::property::{fv, v};
use crate::vcard::property::{parse_property_multiple_values, run_full};

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
            categories: run_full(parse_property_multiple_values(value.as_slice(), ctx.strict))?
                .into_iter()
                .map(|fv| fv.into_inner())
                .collect(),
            params,
        })
    }
}
