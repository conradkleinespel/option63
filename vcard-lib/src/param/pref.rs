use crate::param::ParamTrait;
use crate::parser_internals::result::ParserError;
use crate::property_support::pref::{MAX_PREF, MIN_PREF};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrefParam {
    value: u8,
}

impl ParamTrait for PrefParam {
    fn parse(values: Vec<Vec<u8>>) -> Result<Self, ParserError> {
        let value = values.first().ok_or(ParserError::ParamValue)?;
        let s = std::str::from_utf8(value).map_err(|_| ParserError::ParamValue)?;
        let pref = s.parse::<u8>().map_err(|_| ParserError::ParamValue)?;
        if !(MIN_PREF..=MAX_PREF).contains(&pref) {
            return Err(ParserError::ParamValue);
        }
        Ok(PrefParam { value: pref })
    }
}

impl PrefParam {
    pub fn value(&self) -> u8 {
        self.value
    }

    pub fn new(value: u8) -> Self {
        PrefParam { value }
    }
}
