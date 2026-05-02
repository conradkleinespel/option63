use crate::param::ParamTrait;
use crate::parser_internals::result::ParserError;

// TODO: implement RFC 5646
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LanguageParam {
    Other(String),
}

impl ParamTrait for LanguageParam {
    fn parse(values: Vec<Vec<u8>>) -> Result<Self, ParserError> {
        let value = values.first().ok_or(ParserError::ParamValue)?;
        Ok(LanguageParam::Other(
            String::from_utf8(value.clone()).map_err(|_| ParserError::ParamValue)?,
        ))
    }
}
