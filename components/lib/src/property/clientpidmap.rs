use crate::parser_internals::ParseContext;
use crate::parser_internals::model::Param;
use crate::parser_internals::property_value::parse_property_fields_single_value;
use crate::property_support::PropertyBase;
use crate::{Field, PropertyValueParseError, Value};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientpidmapProperty {
    pid2: u8,
    uri: Vec<u8>,
    params: Vec<Param>,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ClientpidmapError {
    #[error("invalid clientpidmap format")]
    InvalidFormat,
    #[error("property value parse error")]
    InvalidPropertyValue(#[from] PropertyValueParseError),
}

impl ClientpidmapProperty {
    pub fn parse(
        value: Vec<u8>,
        params: Vec<Param>,
        ctx: ParseContext,
    ) -> Result<Self, ClientpidmapError> {
        let values = parse_property_fields_single_value(value.as_slice(), ctx.strict)?;

        if values.len() != 2 {
            return Err(ClientpidmapError::InvalidFormat);
        }

        let pid_str = std::str::from_utf8(values[0].as_slice())
            .map_err(|_| ClientpidmapError::InvalidFormat)?;
        let pid2: u8 = pid_str
            .parse()
            .map_err(|_| ClientpidmapError::InvalidFormat)?;

        if pid2 == 0 {
            return Err(ClientpidmapError::InvalidFormat);
        }

        let uri = values[1].as_slice().to_vec();

        Ok(ClientpidmapProperty { pid2, uri, params })
    }
}

impl PropertyBase for ClientpidmapProperty {
    type Error = ClientpidmapError;

    fn name(&self) -> Vec<u8> {
        b"CLIENTPIDMAP".to_vec()
    }

    fn value(&self) -> Value {
        Value::new(vec![
            Field::new(vec![self.pid2.to_string().into_bytes().into()]),
            Field::new(vec![self.uri.clone().into()]),
        ])
    }

    fn params(&self) -> &[Param] {
        &self.params
    }
}
