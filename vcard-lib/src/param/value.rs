use crate::param::ParamTrait;
use crate::parser_internals::result::ParserError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValueParam {
    Text,
    Uri,
    Date,
    Time,
    DateTime,
    DateAndOrTime,
    Timestamp,
    Boolean,
    Integer,
    Float,
    UtcOffset,
    LanguageTag,
    IanaToken(Vec<u8>),
    XName(Vec<u8>),
}

impl ParamTrait for ValueParam {
    fn parse(values: Vec<Vec<u8>>) -> Result<Self, ParserError> {
        let value = values.first().ok_or(ParserError::ParamValue)?;
        match value.to_ascii_uppercase().as_slice() {
            b"TEXT" => Ok(ValueParam::Text),
            b"URI" => Ok(ValueParam::Uri),
            b"DATE" => Ok(ValueParam::Date),
            b"TIME" => Ok(ValueParam::Time),
            b"DATE-TIME" => Ok(ValueParam::DateTime),
            b"DATE-AND-OR-TIME" => Ok(ValueParam::DateAndOrTime),
            b"TIMESTAMP" => Ok(ValueParam::Timestamp),
            b"BOOLEAN" => Ok(ValueParam::Boolean),
            b"INTEGER" => Ok(ValueParam::Integer),
            b"FLOAT" => Ok(ValueParam::Float),
            b"UTC-OFFSET" => Ok(ValueParam::UtcOffset),
            b"LANGUAGE-TAG" => Ok(ValueParam::LanguageTag),
            v if v.starts_with(b"X-") => Ok(ValueParam::XName(value.clone())),
            _ => Ok(ValueParam::IanaToken(value.clone())),
        }
    }
}
