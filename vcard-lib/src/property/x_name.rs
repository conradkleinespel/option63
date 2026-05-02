use crate::Value;
use crate::parser_internals::ParseContext;
use crate::parser_internals::model::Param;
use crate::property_support::PropertyBase;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenericXNameProperty {
    pub(crate) name: Vec<u8>,
    pub(crate) value: Vec<u8>,
    pub(crate) params: Vec<Param>,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum GenericXNameError {
    #[error("invalid x-name format")]
    InvalidFormat,
}

impl GenericXNameProperty {
    pub fn parse(
        name: Vec<u8>,
        value: Vec<u8>,
        params: Vec<Param>,
        _ctx: ParseContext,
    ) -> Result<Self, GenericXNameError> {
        Ok(Self {
            name,
            value,
            params,
        })
    }
}

impl PropertyBase for GenericXNameProperty {
    type Error = GenericXNameError;

    fn name(&self) -> Vec<u8> {
        self.name.clone()
    }

    fn value(&self) -> Value {
        self.value.clone().into()
    }

    fn params(&self) -> &[Param] {
        &self.params
    }
}
