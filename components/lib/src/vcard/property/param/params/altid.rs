use crate::vcard::parser::ParseError;
use crate::vcard::property::param::ParamTrait;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AltidParam {
    value: Vec<u8>,
}

impl ParamTrait for AltidParam {
    fn parse(values: Vec<Vec<u8>>) -> Result<Self, ParseError> {
        let value = values.first().ok_or(ParseError::ParamValue)?;
        Ok(AltidParam {
            value: value.clone(),
        })
    }
}

impl AltidParam {
    pub fn value(&self) -> &[u8] {
        &self.value
    }
}
