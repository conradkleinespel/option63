use crate::vcard::parser::ParseError;
use crate::vcard::property::param::ParamTrait;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SortAsParam {
    values: Vec<Vec<u8>>,
}

impl ParamTrait for SortAsParam {
    fn parse(values: Vec<Vec<u8>>) -> Result<Self, ParseError> {
        Ok(SortAsParam { values })
    }
}

impl SortAsParam {
    pub fn values(&self) -> &[Vec<u8>] {
        &self.values
    }
}
