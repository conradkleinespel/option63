use crate::vcard::parser::ParseContext;
use crate::vcard::property::PropertyBase;
use crate::vcard::property::Value;
use crate::vcard::property::param::Param;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct XmlProperty {
    value: Vec<u8>,
    params: Vec<Param>,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum XmlError {
    #[error("invalid xml format")]
    InvalidFormat,
}

impl PropertyBase for XmlProperty {
    type Error = XmlError;

    fn name(&self) -> Vec<u8> {
        b"XML".to_vec()
    }

    fn value(&self) -> Value {
        self.value.clone().into()
    }

    fn params(&self) -> &[Param] {
        &self.params
    }
}

impl XmlProperty {
    pub fn parse(value: Vec<u8>, params: Vec<Param>, _ctx: ParseContext) -> Result<Self, XmlError> {
        Ok(XmlProperty { value, params })
    }
}
