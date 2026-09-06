use crate::vcard::parser::ParseError;
use crate::vcard::property::param::ParamTrait;

// TODO: implement RFC 5646
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LanguageParam {
    Other(String),
}

impl ParamTrait for LanguageParam {
    fn parse(values: Vec<Vec<u8>>) -> Result<Self, ParseError> {
        let value = values.first().ok_or(ParseError::ParamValue)?;
        Ok(LanguageParam::Other(
            String::from_utf8(value.clone()).map_err(|_| ParseError::ParamValue)?,
        ))
    }
}
