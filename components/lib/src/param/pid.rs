use crate::param::ParamTrait;
use crate::parser_internals::result::ParserError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PidParam {
    value: Vec<PidParamValue>,
}

impl ParamTrait for PidParam {
    fn parse(values: Vec<Vec<u8>>) -> Result<Self, ParserError> {
        let mut pid_values = Vec::new();
        for value in values {
            let s = std::str::from_utf8(&value).map_err(|_| ParserError::ParamValue)?;
            if let Some((s1, s2)) = s.split_once('.') {
                let v1 = s1.parse::<u32>().map_err(|_| ParserError::ParamValue)?;
                let v2 = s2.parse::<u32>().map_err(|_| ParserError::ParamValue)?;
                pid_values.push(PidParamValue::Double(v1, v2));
            } else {
                let v = s.parse::<u32>().map_err(|_| ParserError::ParamValue)?;
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
