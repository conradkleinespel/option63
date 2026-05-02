use crate::param::ParamTrait;
use crate::parser_internals::result::ParserError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TzParam {
    value: Vec<u8>,
}

impl ParamTrait for TzParam {
    fn parse(values: Vec<Vec<u8>>) -> Result<Self, ParserError> {
        let value = values.first().ok_or(ParserError::ParamValue)?;
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
