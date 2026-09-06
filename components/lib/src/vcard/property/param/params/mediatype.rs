use crate::vcard::parser::ParseError;
use crate::vcard::property::param::ParamTrait;
use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediatypeParam {
    type_name: Vec<u8>,
    subtype_name: Vec<u8>,
    attributes: Vec<HashMap<Vec<u8>, Vec<u8>>>,
}

impl ParamTrait for MediatypeParam {
    fn parse(values: Vec<Vec<u8>>) -> Result<Self, ParseError> {
        // Simplified for now, just take the first value as a placeholder
        // Real implementation would need to parse the mediatype string
        let _value = values.first().ok_or(ParseError::ParamValue)?;
        Ok(MediatypeParam {
            type_name: Vec::new(),
            subtype_name: Vec::new(),
            attributes: Vec::new(),
        })
    }
}
