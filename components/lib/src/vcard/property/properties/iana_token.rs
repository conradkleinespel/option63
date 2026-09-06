use crate::vcard::parser::ParseContext;
use crate::vcard::property::PropertyValueParseError;
use crate::vcard::property::param::Param;
use crate::vcard::property::{PropertyBase, Value};
use crate::vcard::property::{parse_property_single_value, run_full};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenericIanaProperty {
    name: Vec<u8>,
    value: Vec<u8>,
    params: Vec<Param>,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum GenericIanaError {
    #[error("invalid iana token format")]
    InvalidFormat,
    #[error("property value parse error")]
    InvalidPropertyValue(#[from] PropertyValueParseError),
}

impl GenericIanaProperty {
    pub fn parse(
        name: Vec<u8>,
        value: Vec<u8>,
        params: Vec<Param>,
        ctx: ParseContext,
    ) -> Result<Self, GenericIanaError> {
        Ok(Self {
            name,
            value: run_full(parse_property_single_value(value.as_slice(), ctx.strict))?
                .into_inner(),
            params,
        })
    }
}

impl PropertyBase for GenericIanaProperty {
    type Error = GenericIanaError;

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
