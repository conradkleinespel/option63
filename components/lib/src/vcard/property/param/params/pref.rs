use crate::vcard::parser::ParseError;
use crate::vcard::property::param::ParamTrait;
use crate::vcard::property::{MAX_PREF, MIN_PREF};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrefParam {
    value: u8,
}

impl ParamTrait for PrefParam {
    fn parse(values: Vec<Vec<u8>>) -> Result<Self, ParseError> {
        let value = values.first().ok_or(ParseError::ParamValue)?;
        let s = std::str::from_utf8(value).map_err(|_| ParseError::ParamValue)?;
        let pref = s.parse::<u8>().map_err(|_| ParseError::ParamValue)?;
        if !(MIN_PREF..=MAX_PREF).contains(&pref) {
            return Err(ParseError::ParamValue);
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
