// A property's value must be parsed according to the property's schema:
//
// 1. Structural Splitting (Compounds)
// - If the property is compound (e.g., N, ADR), split the payload on UNESCAPED ";"
// - If the property is simple (e.g., NOTE), treat the whole payload as a single field.
// 2. Multi-value Splitting (Lists)
// - For each resulting field, if the schema allows multiple values,
//   split the field on UNESCAPED ","
// 3. Final Unescaping (Clean up)
// - For every final extracted value, apply the unescaping rules:
//     - replace "\;" with ";"
//     - replace "\," with ","
//     - replace "\n" and "\N" with an actual newline character
//     - replace "\\" with "\"

use crate::parser_internals::utils::is_escaped;
use crate::{Field, FieldValue, PropertyValueParseError, fv};

/// Parses a vCard value made of single-value fields, like the ADR property
pub fn parse_property_fields_single_value(
    input: &[u8],
    strict: bool,
) -> Result<Vec<FieldValue>, PropertyValueParseError> {
    let fields = parse_property_fields_and_values(input, true, false, strict)?;

    let mut flattened_values = Vec::new();

    for field in fields {
        if field.values().len() != 1 {
            return Err(PropertyValueParseError::InvalidNumberFieldValues(
                field.values().len(),
            ));
        }
        for value in field.values() {
            flattened_values.push(value.clone());
        }
    }

    Ok(flattened_values)
}

/// Parses a vCard value made of a single value, like the EMAIL property
pub fn parse_property_single_value(
    input: &[u8],
    strict: bool,
) -> Result<FieldValue, PropertyValueParseError> {
    let fields = parse_property_fields_and_values(input, false, false, strict)?;

    if fields.len() != 1 {
        return Err(PropertyValueParseError::InvalidNumberFields(fields.len()));
    }

    for field in fields.iter() {
        if field.values().len() != 1 {
            return Err(PropertyValueParseError::InvalidNumberFieldValues(
                field.values().len(),
            ));
        }
    }

    Ok(fields[0].values()[0].clone())
}

/// Parses a vCard value made of multiple comma-separated values, like the CATEGORIES property
pub fn parse_property_multiple_values(
    input: &[u8],
    strict: bool,
) -> Result<Vec<FieldValue>, PropertyValueParseError> {
    let fields = parse_property_fields_and_values(input, false, true, strict)?;

    if fields.len() != 1 {
        return Err(PropertyValueParseError::InvalidNumberFields(fields.len()));
    }

    Ok(fields[0].values().to_vec())
}

pub fn parse_property_fields_and_values(
    input: &[u8],
    compound: bool,
    multiple_values: bool,
    strict: bool,
) -> Result<Vec<Field>, PropertyValueParseError> {
    let fields: Vec<Field> = if compound {
        split_multiple_fields(input, multiple_values)
    } else {
        if multiple_values {
            vec![Field::new(split_multiple_values(input))]
        } else {
            vec![Field::new(vec![fv!(input.to_vec())])]
        }
    };

    let mut out_fields = Vec::new();
    for field in fields {
        let mut out_values = Vec::new();
        for value in field.values() {
            let unescaped = unescape(value.as_slice(), strict)?;
            out_values.push(fv!(unescaped));
        }
        out_fields.push(Field::new(out_values));
    }

    Ok(out_fields)
}

fn split_multiple_fields(input: &[u8], multiple_values: bool) -> Vec<Field> {
    let mut fields: Vec<Field> = vec![];
    for field in split_on_char(input, b';') {
        if multiple_values {
            fields.push(Field::new(split_multiple_values(field.as_slice())));
        } else {
            fields.push(Field::new(vec![fv!(field)]));
        }
    }
    fields
}

fn split_multiple_values(input: &[u8]) -> Vec<FieldValue> {
    split_on_char(input, b',')
        .iter()
        .map(|v| fv!(v.to_vec()))
        .collect()
}

fn split_on_char(input: &[u8], byte: u8) -> Vec<Vec<u8>> {
    let mut parts = vec![];
    let mut current_part = vec![];
    for (i, b) in input.iter().enumerate() {
        if b == &byte && !is_escaped(input, i) {
            // create a new part
            parts.push(current_part);
            current_part = vec![];
        } else {
            // continue adding to existing part
            current_part.push(*b)
        }
    }

    parts.push(current_part);

    parts
}

fn unescape(input: &[u8], strict: bool) -> Result<Vec<u8>, PropertyValueParseError> {
    let mut out = vec![];

    let mut i = 0;
    while i < input.len() {
        let b0 = input[i];

        // Not escaped character
        if b0 != b'\\' {
            out.push(b0);
            i += 1;
            continue;
        }

        // Backslash not followed by anything else, shouldn't happen but handle gracefully
        // by ignoring it
        if i + 1 == input.len() {
            if strict {
                return Err(PropertyValueParseError::InvalidEscapeCharacters);
            }
            break;
        }

        // Escaped character can be \\ \, \; \n \N
        let b1 = input[i + 1];
        match b1 {
            b'\\' | b',' | b';' => out.push(b1),
            b'n' | b'N' => out.push(b'\n'),
            _ => {
                // Invalid escape characters are ignored unless we want total RFC compliance
                if strict {
                    return Err(PropertyValueParseError::InvalidEscapeCharacters);
                }
            }
        }

        i += 2;
    }

    Ok(out)
}

pub fn escape(input: &[u8], escape_field_separator: bool, escape_value_separator: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len() * 2);
    for c in input {
        match c {
            b'\\' => {
                out.push(b'\\');
                out.push(*c)
            }
            b';' => {
                if escape_field_separator {
                    out.push(b'\\');
                }
                out.push(*c)
            }
            b',' => {
                if escape_value_separator {
                    out.push(b'\\');
                }
                out.push(*c)
            }
            b'\n' => {
                out.push(b'\\');
                out.push(b'n')
            }
            c => out.push(*c),
        }
    }
    out
}
