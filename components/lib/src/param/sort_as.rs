use crate::param::ParamTrait;
use crate::parser_internals::result::ParserError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SortAsParam {
    values: Vec<Vec<u8>>,
}

impl ParamTrait for SortAsParam {
    fn parse(values: Vec<Vec<u8>>) -> Result<Self, ParserError> {
        Ok(SortAsParam { values })
    }
}

impl SortAsParam {
    pub fn values(&self) -> &[Vec<u8>] {
        &self.values
    }
}
