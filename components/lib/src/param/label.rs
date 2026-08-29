use crate::param::ParamTrait;
use crate::parser_internals::result::ParserError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LabelParam {
    value: Vec<u8>,
}

impl ParamTrait for LabelParam {
    fn parse(values: Vec<Vec<u8>>) -> Result<Self, ParserError> {
        let value = values.first().ok_or(ParserError::ParamValue)?;
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
