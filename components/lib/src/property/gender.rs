use crate::parser_internals::ParseContext;
use crate::parser_internals::model::Param;
use crate::parser_internals::property_value::parse_property_fields_single_value;
use crate::property_support::PropertyBase;
use crate::{PropertyValueParseError, Value, f, fv, v};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenderProperty {
    sex: Sex,
    identity: Option<Vec<u8>>,
    params: Vec<Param>,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum GenderError {
    #[error("invalid gender sex character")]
    InvalidSex,
    #[error("invalid number of fields, expected 2, got {0}")]
    InvalidNumberFields(usize),
    #[error("property value parse error")]
    InvalidPropertyValue(#[from] PropertyValueParseError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Sex {
    Male,
    Female,
    Other,
    None,
    Unknown,
    Empty,
}

impl PropertyBase for GenderProperty {
    type Error = GenderError;

    fn name(&self) -> Vec<u8> {
        b"GENDER".to_vec()
    }

    fn value(&self) -> Value {
        if let Some(identity) = &self.identity {
            if let Some(sex_char) = self.sex.to_char() {
                v!(f!(fv!(sex_char)), f!(fv!(identity)))
            } else {
                v!(f!(fv!("")), f!(fv!(identity)))
            }
        } else {
            if let Some(sex_char) = self.sex.to_char() {
                v!(f!(fv!(sex_char)))
            } else {
                v!(f!(fv!("")))
            }
        }
    }

    fn params(&self) -> &[Param] {
        &self.params
    }
}

impl GenderProperty {
    pub fn parse(
        input: Vec<u8>,
        params: Vec<Param>,
        ctx: ParseContext,
    ) -> Result<Self, GenderError> {
        let values = parse_property_fields_single_value(input.as_slice(), ctx.strict)?;

        if values.len() > 2 {
            return Err(GenderError::InvalidNumberFields(values.len()));
        }

        let sex_part = values
            .first()
            .map(|v| v.as_slice().to_vec())
            .unwrap_or(vec![]);
        let sex = if sex_part.is_empty() {
            Sex::Empty
        } else {
            match sex_part.as_slice().iter().next() {
                Some(c) => Sex::from_char(*c).ok_or(GenderError::InvalidSex)?,
                None => Sex::Empty,
            }
        };

        let identity = values.get(1).map(|v| v.as_slice().to_vec());

        Ok(GenderProperty {
            sex,
            identity,
            params,
        })
    }

    /// Get the sex component
    pub fn sex(&self) -> Sex {
        self.sex
    }

    /// Get the identity component
    pub fn identity(&self) -> Option<&[u8]> {
        self.identity.as_deref()
    }
}

impl Sex {
    fn from_char(c: u8) -> Option<Self> {
        match c {
            b'M' => Some(Sex::Male),
            b'F' => Some(Sex::Female),
            b'O' => Some(Sex::Other),
            b'N' => Some(Sex::None),
            b'U' => Some(Sex::Unknown),
            _ => None,
        }
    }

    fn to_char(self) -> Option<char> {
        match self {
            Sex::Male => Some('M'),
            Sex::Female => Some('F'),
            Sex::Other => Some('O'),
            Sex::None => Some('N'),
            Sex::Unknown => Some('U'),
            Sex::Empty => None,
        }
    }
}
