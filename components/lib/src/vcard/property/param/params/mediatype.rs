use crate::vcard::parser::ParseError;
use crate::vcard::property::param::ParamTrait;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediatypeParam {
    raw: Vec<u8>,
}

impl MediatypeParam {
    fn validate(raw: &[u8]) -> Result<(), ParseError> {
        // mediatype = type-name "/" subtype-name *( *WSP ";" *WSP mediatype-param )
        let Some(slash) = raw.iter().position(|b| b == &b'/') else {
            return Err(ParseError::ParamValue);
        };
        let type_name = &raw[..slash];
        let rest = &raw[slash + 1..];

        if type_name.is_empty() || rest.is_empty() {
            return Err(ParseError::ParamValue);
        }

        let (subtype_name, parameters) = match rest.iter().position(|b| b == &b';') {
            Some(semi) => (&rest[..semi], &rest[semi + 1..]),
            None => (rest, &rest[rest.len()..]),
        };

        if subtype_name.is_empty()
            || !type_name
                .iter()
                .chain(subtype_name.iter())
                .all(|b| is_type_name_byte(*b))
            || subtype_name.contains(&b'/')
        {
            return Err(ParseError::ParamValue);
        }

        // TODO: validate individual mediatype-params after the first ";".
        let _ = parameters;

        Ok(())
    }
}

fn is_type_name_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric()
        || matches!(
            b,
            b'!' | b'#' | b'$' | b'&' | b'-' | b'^' | b'_' | b'.' | b'+'
        )
}

impl ParamTrait for MediatypeParam {
    fn parse(values: Vec<Vec<u8>>) -> Result<Self, ParseError> {
        let raw = values.first().ok_or(ParseError::ParamValue)?;
        Self::validate(raw)?;
        Ok(MediatypeParam { raw: raw.clone() })
    }
}

impl MediatypeParam {
    pub fn raw(&self) -> &[u8] {
        &self.raw
    }
}
