use crate::param::ParamTrait;
use crate::parser_internals::result::ParserError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CalscaleParam {
    Gregorian,
    IanaToken(Vec<u8>),
    XName(Vec<u8>),
}

impl ParamTrait for CalscaleParam {
    fn parse(values: Vec<Vec<u8>>) -> Result<Self, ParserError> {
        let value = values.first().ok_or(ParserError::ParamValue)?;
        match value.to_ascii_uppercase().as_slice() {
            b"GREGORIAN" => Ok(CalscaleParam::Gregorian),
            v if v.starts_with(b"X-") => Ok(CalscaleParam::XName(value.clone())),
            _ => Ok(CalscaleParam::IanaToken(value.clone())),
        }
    }
}
