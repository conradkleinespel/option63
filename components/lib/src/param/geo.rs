use crate::param::ParamTrait;
use crate::parser_internals::result::ParserError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeoParam {
    uri: Vec<u8>,
}

impl ParamTrait for GeoParam {
    fn parse(values: Vec<Vec<u8>>) -> Result<Self, ParserError> {
        let value = values.first().ok_or(ParserError::ParamValue)?;
        Ok(GeoParam { uri: value.clone() })
    }
}

impl GeoParam {
    pub fn uri(&self) -> &[u8] {
        &self.uri
    }
}
