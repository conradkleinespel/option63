use crate::vcard::parser::{ParseError, R, RE};
use crate::vcard::property::{Field, FieldValue, PropertyValueParseError, fv};
use nom::Parser;
use nom::bytes::complete::{escaped, tag, take, take_while1};
use nom::combinator::{all_consuming, opt};
use nom::multi::{many0, separated_list0};

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

/// Runs a full-consuming parser, expecting the whole input to be consumed.
/// Returns the parsed value, or an error if the parser failed or left input
/// unconsumed.
pub(crate) fn run_full<T>(
    result: RE<T, PropertyValueParseError>,
) -> Result<T, PropertyValueParseError> {
    match result {
        Ok(([], value)) => Ok(value),
        _ => Err(PropertyValueParseError::InvalidEscapeCharacters),
    }
}

/// Parses a vCard value made of single-value fields, like the ADR property
pub(crate) fn parse_property_fields_single_value(
    input: &[u8],
    strict: bool,
) -> RE<'_, Vec<FieldValue>, PropertyValueParseError> {
    let (_, fields) = parse_property_fields_and_values(input, true, false, strict)?;

    let mut flattened_values = Vec::new();

    for field in fields {
        if field.values().len() != 1 {
            return Err(nom::Err::Error(
                PropertyValueParseError::InvalidNumberFieldValues(field.values().len()),
            ));
        }
        for value in field.values() {
            flattened_values.push(value.clone());
        }
    }

    Ok((b"", flattened_values))
}

/// Parses a vCard value made of a single value, like the EMAIL property
pub(crate) fn parse_property_single_value(
    input: &[u8],
    strict: bool,
) -> RE<'_, FieldValue, PropertyValueParseError> {
    let (_, fields) = parse_property_fields_and_values(input, false, false, strict)?;

    if fields.len() != 1 {
        return Err(nom::Err::Error(
            PropertyValueParseError::InvalidNumberFields(fields.len()),
        ));
    }

    for field in fields.iter() {
        if field.values().len() != 1 {
            return Err(nom::Err::Error(
                PropertyValueParseError::InvalidNumberFieldValues(field.values().len()),
            ));
        }
    }

    Ok((b"", fields[0].values()[0].clone()))
}

/// Parses a vCard value made of multiple comma-separated values, like the CATEGORIES property
pub(crate) fn parse_property_multiple_values(
    input: &[u8],
    strict: bool,
) -> RE<'_, Vec<FieldValue>, PropertyValueParseError> {
    let (_, fields) = parse_property_fields_and_values(input, false, true, strict)?;

    if fields.len() != 1 {
        return Err(nom::Err::Error(
            PropertyValueParseError::InvalidNumberFields(fields.len()),
        ));
    }

    Ok((b"", fields[0].values().to_vec()))
}

pub(crate) fn parse_property_fields_and_values(
    input: &[u8],
    compound: bool,
    multiple_values: bool,
    strict: bool,
) -> RE<'_, Vec<Field>, PropertyValueParseError> {
    let (_, fields) = all_consuming(value_sequence(compound, multiple_values))
        .parse_complete(input)
        .map_err(|_| nom::Err::Error(PropertyValueParseError::InvalidEscapeCharacters))?;

    let mut out_fields = Vec::new();
    for field in fields {
        let mut out_values = Vec::new();
        for value in field {
            let unescaped = unescape(value, strict)
                .map_err(|_| nom::Err::Error(PropertyValueParseError::InvalidEscapeCharacters))?;
            out_values.push(fv!(unescaped));
        }
        out_fields.push(Field::new(out_values));
    }

    Ok((b"", out_fields))
}

/// Parses the structural split of a property value, before unescaping, into a
/// list of fields, each itself a list of raw (still escaped) value slices.
fn value_sequence(
    compound: bool,
    multiple_values: bool,
) -> impl FnMut(&[u8]) -> R<'_, Vec<Vec<&[u8]>>> {
    move |input: &[u8]| {
        if compound {
            let (rest, fields) = separated(input, b';')?;
            if multiple_values {
                let mut out = Vec::with_capacity(fields.len());
                for f in fields {
                    let (_, subfields) = separated(f, b',')?;
                    out.push(subfields);
                }
                Ok((rest, out))
            } else {
                Ok((rest, fields.into_iter().map(|f| vec![f]).collect()))
            }
        } else if multiple_values {
            let (rest, values) = separated(input, b',')?;
            Ok((rest, vec![values]))
        } else {
            Ok((&input[input.len()..], vec![vec![input]]))
        }
    }
}

/// Splits `input` on UNESCAPED `sep` bytes, preserving `\x` escape pairs. The
/// segment parser matches empty, so a trailing separator yields a trailing
/// empty part (`"a,b,"` -> `["a", "b", ""]`).
fn separated(input: &[u8], sep: u8) -> R<'_, Vec<&[u8]>> {
    separated_list0(tag(&[sep][..]), escaped_segment(sep)).parse_complete(input)
}

/// A parser that consumes an escaped-aware segment of input, stopping at (but
/// not consuming) the first UNESCAPED `sep` byte. An escaped byte (`\x`) is
/// taken together so that it is never read as a separator.
fn escaped_segment(sep: u8) -> impl FnMut(&[u8]) -> R<'_, &[u8]> {
    move |input: &[u8]| {
        let esc = escaped(take_while1(|b| b != sep && b != b'\\'), '\\', take(1usize));
        let parsed: R<'_, Option<&[u8]>> = opt(esc).parse_complete(input);
        match parsed {
            Ok((rest, Some(seg))) => Ok((rest, seg)),
            Ok((rest, None)) => Ok((rest, &rest[..0])),
            Err(e) => Err(e),
        }
    }
}

/// Parses a single unit of a value: either a run of plain characters or a
/// backslash escape sequence. Invalid escapes are dropped in lax mode and
/// rejected in strict mode; a lone trailing backslash is dropped in lax mode
/// and rejected in strict mode.
pub(crate) fn escape_unit(strict: bool) -> impl FnMut(&[u8]) -> R<'_, Vec<u8>> {
    move |input: &[u8]| {
        if input.is_empty() {
            return Err(nom::Err::Error(ParseError::Generic));
        }

        if input[0] != b'\\' {
            let parsed: R<'_, &[u8]> = take_while1(|b: u8| b != b'\\').parse_complete(input);
            return match parsed {
                Ok((rest, plain)) => Ok((rest, plain.to_vec())),
                Err(_) => Err(nom::Err::Error(ParseError::Generic)),
            };
        }

        if input.len() < 2 {
            // Lone trailing backslash.
            if strict {
                Err(nom::Err::Error(ParseError::Generic))
            } else {
                Ok((&input[1..], Vec::new()))
            }
        } else {
            let esc = input[1];
            match esc {
                b'\\' | b',' | b';' => Ok((&input[2..], vec![esc])),
                b'n' | b'N' => Ok((&input[2..], vec![b'\n'])),
                _ if strict => Err(nom::Err::Error(ParseError::Generic)),
                _ => Ok((&input[2..], Vec::new())),
            }
        }
    }
}

pub(crate) fn unescape(input: &[u8], strict: bool) -> Result<Vec<u8>, PropertyValueParseError> {
    let (rest, units) = many0(escape_unit(strict))
        .parse_complete(input)
        .map_err(|_| PropertyValueParseError::InvalidEscapeCharacters)?;
    if !rest.is_empty() {
        return Err(PropertyValueParseError::InvalidEscapeCharacters);
    }
    Ok(units.into_iter().flatten().collect())
}

pub(crate) fn escape(
    input: &[u8],
    escape_field_separator: bool,
    escape_value_separator: bool,
) -> Vec<u8> {
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

pub(crate) fn param_value_needs_quoting(input: &[u8]) -> bool {
    // Input can contain only safe-chars of qsafe-chars, but if we find stuff that's not qsafe,
    // that is ":" (0x3A) or ";" (0x3B), we'll need to quote the value
    input.iter().any(|c| matches!(c, 0x3A | 0x3B))
}
