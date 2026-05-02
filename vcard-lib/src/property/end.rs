use crate::Value;
use crate::parser_internals::ParseContext;
use crate::parser_internals::model::Param;
use crate::property_support::PropertyBase;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EndProperty {
    params: Vec<Param>,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum EndError {
    #[error("invalid end value, must be VCARD")]
    InvalidValue,
    #[error("parameters not allowed on END property")]
    ParametersNotAllowed,
}

impl PropertyBase for EndProperty {
    type Error = EndError;

    fn name(&self) -> Vec<u8> {
        b"END".to_vec()
    }

    fn value(&self) -> Value {
        b"VCARD".to_vec().into()
    }

    fn params(&self) -> &[Param] {
        &self.params
    }
}

impl EndProperty {
    pub fn parse(value: Vec<u8>, params: Vec<Param>, _ctx: ParseContext) -> Result<Self, EndError> {
        if !params.is_empty() {
            return Err(EndError::ParametersNotAllowed);
        }

        let value_upper = value.to_ascii_uppercase();
        if value_upper != b"VCARD" {
            return Err(EndError::InvalidValue);
        }

        Ok(EndProperty { params })
    }
}
