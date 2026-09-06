use crate::vcard::parser::ParseError;
use crate::vcard::property::param::ParamTrait;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PidParam {
    value: Vec<PidParamValue>,
}

impl ParamTrait for PidParam {
    fn parse(values: Vec<Vec<u8>>) -> Result<Self, ParseError> {
        let mut pid_values = Vec::new();
        for value in values {
            let s = std::str::from_utf8(&value).map_err(|_| ParseError::ParamValue)?;
            if let Some((s1, s2)) = s.split_once('.') {
                let v1 = s1.parse::<u32>().map_err(|_| ParseError::ParamValue)?;
                let v2 = s2.parse::<u32>().map_err(|_| ParseError::ParamValue)?;
                pid_values.push(PidParamValue::Double(v1, v2));
            } else {
                let v = s.parse::<u32>().map_err(|_| ParseError::ParamValue)?;
                pid_values.push(PidParamValue::Single(v));
            }
        }
        Ok(PidParam { value: pid_values })
    }
}

impl PidParam {
    pub fn value(&self) -> &[PidParamValue] {
        &self.value
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PidParamValue {
    Single(u32),
    Double(u32, u32),
}
