use crate::parser_internals::ParseContext;
use crate::parser_internals::model::Param;
use crate::parser_internals::property_value::parse_property_fields_single_value;
use crate::property_support::PropertyBase;
use crate::property_support::pref::PropertyPref;
use crate::{PropertyValueParseError, Value, f, fv, v};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdrProperty {
    street_address: Vec<u8>,
    locality: Vec<u8>,
    region: Vec<u8>,
    postal_code: Vec<u8>,
    country: Vec<u8>,
    params: Vec<Param>,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum AdrError {
    #[error("invalid adr component count: {0}")]
    InvalidComponentCount(usize),
    #[error("property value parse error")]
    InvalidPropertyValue(#[from] PropertyValueParseError),
}

impl PropertyBase for AdrProperty {
    type Error = AdrError;

    fn name(&self) -> Vec<u8> {
        b"ADR".to_vec()
    }

    fn value(&self) -> Value {
        // Post office box, extended address and street address are merged to increase
        // interoperability with VCard 3.0, separated by commas, as described in RFC
        v!(
            f!(fv!("")),
            f!(fv!("")),
            f!(fv!(self.street_address.as_slice())),
            f!(fv!(self.locality.as_slice())),
            f!(fv!(self.region.as_slice())),
            f!(fv!(self.postal_code.as_slice())),
            f!(fv!(self.country.as_slice()))
        )
    }

    fn params(&self) -> &[Param] {
        &self.params
    }
}

impl PropertyPref for AdrProperty {}

impl AdrProperty {
    pub fn parse(value: Vec<u8>, params: Vec<Param>, ctx: ParseContext) -> Result<Self, AdrError> {
        let components = parse_property_fields_single_value(value.as_slice(), ctx.strict)?;

        match components.len() {
            // VCard 3.0 files sometimes do not specify post office box and/or extended address
            5 => Ok(AdrProperty {
                street_address: components[0].as_slice().to_vec(),
                locality: components[1].as_slice().to_vec(),
                region: components[2].as_slice().to_vec(),
                postal_code: components[3].as_slice().to_vec(),
                country: components[4].as_slice().to_vec(),
                params,
            }),
            // VCard 3.0 files sometimes do not specify post office box and/or extended address
            6 => Ok(AdrProperty {
                // For VCard 3.0 compat, do not use post office box and extended address
                street_address: vec![
                    components[0].as_slice().to_vec(),
                    components[1].as_slice().to_vec(),
                ]
                .into_iter()
                .filter(|s| !s.is_empty())
                .collect::<Vec<Vec<u8>>>()
                .join(b", ".as_slice()),
                locality: components[2].as_slice().to_vec(),
                region: components[3].as_slice().to_vec(),
                postal_code: components[4].as_slice().to_vec(),
                country: components[5].as_slice().to_vec(),
                params,
            }),
            7 => Ok(AdrProperty {
                // For VCard 3.0 compat, do not use post office box and extended address
                street_address: vec![
                    components[0].as_slice().to_vec(),
                    components[1].as_slice().to_vec(),
                    components[2].as_slice().to_vec(),
                ]
                .into_iter()
                .filter(|s| !s.is_empty())
                .collect::<Vec<Vec<u8>>>()
                .join(b", ".as_slice()),
                locality: components[3].as_slice().to_vec(),
                region: components[4].as_slice().to_vec(),
                postal_code: components[5].as_slice().to_vec(),
                country: components[6].as_slice().to_vec(),
                params,
            }),
            _ => Err(AdrError::InvalidComponentCount(components.len())),
        }
    }

    /// Get the street address component
    pub fn street_address(&self) -> &[u8] {
        &self.street_address
    }

    /// Get the locality (city) component
    pub fn locality(&self) -> &[u8] {
        &self.locality
    }

    /// Get the region (state/province) component
    pub fn region(&self) -> &[u8] {
        &self.region
    }

    /// Get the postal code component
    pub fn postal_code(&self) -> &[u8] {
        &self.postal_code
    }

    /// Get the country component
    pub fn country(&self) -> &[u8] {
        &self.country
    }
}
