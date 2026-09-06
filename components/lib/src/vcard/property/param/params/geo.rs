use crate::vcard::parser::ParseError;
use crate::vcard::property::param::ParamTrait;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeoParam {
    uri: Vec<u8>,
}

impl ParamTrait for GeoParam {
    fn parse(values: Vec<Vec<u8>>) -> Result<Self, ParseError> {
        let value = values.first().ok_or(ParseError::ParamValue)?;
        Ok(GeoParam { uri: value.clone() })
    }
}

impl GeoParam {
    pub fn uri(&self) -> &[u8] {
        &self.uri
    }
}
