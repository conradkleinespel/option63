use crate::vcard::parser::ParseError;
use crate::vcard::property::param::ParamTrait;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LabelParam {
    value: Vec<u8>,
}

impl ParamTrait for LabelParam {
    fn parse(values: Vec<Vec<u8>>) -> Result<Self, ParseError> {
        let value = values.first().ok_or(ParseError::ParamValue)?;
        Ok(LabelParam {
            value: value.clone(),
        })
    }
}

impl LabelParam {
    pub fn value(&self) -> &[u8] {
        &self.value
    }

    pub fn new(value: Vec<u8>) -> Self {
        LabelParam { value }
    }
}
