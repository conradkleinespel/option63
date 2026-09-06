use crate::vcard::parser::ParseError;
use crate::vcard::property::param::ParamTrait;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TzParam {
    value: Vec<u8>,
}

impl ParamTrait for TzParam {
    fn parse(values: Vec<Vec<u8>>) -> Result<Self, ParseError> {
        let value = values.first().ok_or(ParseError::ParamValue)?;
        Ok(TzParam {
            value: value.clone(),
        })
    }
}

impl TzParam {
    pub fn value(&self) -> &[u8] {
        &self.value
    }
}
