use crate::Value;
use crate::parser_internals::ParseContext;
use crate::parser_internals::model::Param;
use crate::property_support::PropertyBase;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelatedProperty {
    value: Vec<u8>,
    params: Vec<Param>,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum RelatedError {
    #[error("invalid related format")]
    InvalidFormat,
}

impl PropertyBase for RelatedProperty {
    type Error = RelatedError;

    fn name(&self) -> Vec<u8> {
        b"RELATED".to_vec()
    }

    fn value(&self) -> Value {
        self.value.clone().into()
    }

    fn params(&self) -> &[Param] {
        &self.params
    }
}

impl RelatedProperty {
    pub fn parse(
        value: Vec<u8>,
        params: Vec<Param>,
        _ctx: ParseContext,
    ) -> Result<Self, RelatedError> {
        Ok(RelatedProperty { value, params })
    }
}
