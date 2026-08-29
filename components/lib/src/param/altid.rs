use crate::param::ParamTrait;
use crate::parser_internals::result::ParserError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AltidParam {
    value: Vec<u8>,
}

impl ParamTrait for AltidParam {
    fn parse(values: Vec<Vec<u8>>) -> Result<Self, ParserError> {
        let value = values.first().ok_or(ParserError::ParamValue)?;
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
