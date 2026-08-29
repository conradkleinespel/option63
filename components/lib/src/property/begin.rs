use crate::Value;
use crate::parser_internals::ParseContext;
use crate::parser_internals::model::Param;
use crate::property_support::PropertyBase;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BeginProperty {
    params: Vec<Param>,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum BeginError {
    #[error("invalid begin value, must be VCARD")]
    InvalidValue,
    #[error("parameters not allowed on BEGIN property")]
    ParametersNotAllowed,
}

impl PropertyBase for BeginProperty {
    type Error = BeginError;

    fn name(&self) -> Vec<u8> {
        b"BEGIN".to_vec()
    }

    fn value(&self) -> Value {
        b"VCARD".to_vec().into()
    }

    fn params(&self) -> &[Param] {
        &self.params
    }
}

impl BeginProperty {
    pub fn parse(
        value: Vec<u8>,
        params: Vec<Param>,
        _ctx: ParseContext,
    ) -> Result<Self, BeginError> {
        if !params.is_empty() {
            return Err(BeginError::ParametersNotAllowed);
        }

        let value_upper = value.to_ascii_uppercase();
        if value_upper != b"VCARD" {
            return Err(BeginError::InvalidValue);
        }

        Ok(BeginProperty { params })
    }
}
