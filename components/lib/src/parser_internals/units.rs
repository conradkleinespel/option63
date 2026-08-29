use crate::parser_internals::result::{ParserError, ParserOutput, ParserResult};
use crate::parser_internals::utils::{split_at_matched, utf8_width_from_first_byte};

/// 1*(ALPHA / DIGIT / "-"), used by group, iana-token, and others
fn parse_1x_alpha_digit_dash(input: &[u8]) -> ParserResult<'_, ()> {
    // Parse one or more ALPHA, DIGIT, or "-"
    let mut matched_len = 0;
    let mut remaining = input;

    while let Ok(out_alpha_digit_or_dash) = parse_alpha(remaining)
        .or_else(|_| parse_digit(remaining))
        .or_else(|_| parse_dash(remaining))
    {
        matched_len += out_alpha_digit_or_dash.matched().len();
        remaining = out_alpha_digit_or_dash.remaining();
    }

    if matched_len > 0 {
        split_at_matched(input, matched_len)
    } else {
        Err(ParserError::Generic)
    }
}

/// Parse a group identifier
/// group = 1*(ALPHA / DIGIT / "-")
pub(crate) fn parse_group(input: &[u8]) -> ParserResult<'_, ()> {
    parse_1x_alpha_digit_dash(input)
}

/// Parse an IANA token
/// iana-token = 1*(ALPHA / DIGIT / "-")
fn parse_iana_token(input: &[u8]) -> ParserResult<'_, ()> {
    parse_1x_alpha_digit_dash(input)
}

/// Parse an x-name
/// x-name = "x-" 1*(ALPHA / DIGIT / "-")
fn parse_x_name(input: &[u8]) -> ParserResult<'_, ()> {
    if input.len() < 3 || !input.starts_with(b"x-") {
        return Err(ParserError::Generic);
    }

    let out_prefix = split_at_matched(input, 2)?;
    let out_token = parse_1x_alpha_digit_dash(out_prefix.remaining())?;

    let full_match = &input[..out_prefix.matched().len() + out_token.matched().len()];
    Ok(ParserOutput::new(full_match, out_token.remaining()))
}

pub(crate) fn parse_property_name(input: &[u8]) -> ParserResult<'_, ()> {
    if let Ok(out) = parse_x_name(input) {
        return Ok(ParserOutput::with_output(
            out.matched(),
            out.remaining(),
            (),
        ));
    }

    if let Ok(out) = parse_iana_token(input) {
        return Ok(ParserOutput::with_output(
            out.matched(),
            out.remaining(),
            (),
        ));
    }

    Err(ParserError::Generic)
}

/// Parse a parameter name
/// param-name = iana-token / x-name
pub(crate) fn parse_param_name(input: &[u8]) -> ParserResult<'_, ()> {
    let out = parse_x_name(input).or_else(|_| parse_iana_token(input))?;
    Ok(ParserOutput::new(out.matched(), out.remaining()))
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParamValue {
    value: Vec<u8>,
    quoted: bool,
}

impl ParamValue {
    fn new(value: Vec<u8>, quoted: bool) -> Self {
        Self { value, quoted }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.value
    }
}

/// Parse a parameter value
/// param-value = *SAFE-CHAR / DQUOTE *QSAFE-CHAR DQUOTE
pub(crate) fn parse_param_value(input: &[u8]) -> (&[u8], &[u8], ParamValue) {
    // Try unquoted value: *SAFE-CHAR (until comma or semicolon)
    let mut matched_len = 0;
    let mut remaining = input;

    while let Ok(out_safe_char) = parse_safe_char(remaining) {
        // Per RFC:
        // Property parameter value elements that contain the COLON (U+003A),
        // SEMICOLON (U+003B), or COMMA (U+002C) character separators MUST be
        // specified as quoted-string text values.
        if parse_colon(remaining).is_ok()
            || parse_semicolon(remaining).is_ok()
            || parse_comma(remaining).is_ok()
        {
            break;
        }
        matched_len += out_safe_char.matched().len();
        remaining = out_safe_char.remaining();
    }

    if matched_len > 0 {
        return (
            &input[..matched_len],
            remaining,
            ParamValue::new(input[..matched_len].to_vec(), false),
        );
    }

    // Try quoted value: DQUOTE *QSAFE-CHAR DQUOTE
    if let Ok(out_open) = parse_dquote(input) {
        let mut matched_len = 0;
        let mut remaining = out_open.remaining();

        while let Ok(out_qsafe_char) = parse_qsafe_char(remaining) {
            matched_len += out_qsafe_char.matched().len();
            remaining = out_qsafe_char.remaining();
        }

        if let Ok(out_close) = parse_dquote(remaining) {
            let full_match = &input[..1 + matched_len + 1];
            return (
                full_match,
                out_close.remaining(),
                ParamValue::new(full_match[1..full_match.len() - 1].to_vec(), true),
            );
        }
    }

    // Allow empty values
    (b"", input, ParamValue::new(vec![], false))
}

/// COMMA = %x2C
pub(crate) fn parse_comma(input: &[u8]) -> ParserResult<'_, ()> {
    match input.iter().next() {
        Some(b',') => split_at_matched(input, 1),
        _ => Err(ParserError::Generic),
    }
}

/// DASH = -
fn parse_dash(input: &[u8]) -> ParserResult<'_, ()> {
    match input.iter().next() {
        Some(b'-') => split_at_matched(input, 1),
        _ => Err(ParserError::Generic),
    }
}

/// BACKSLASH = %x5C
#[allow(unused)]
fn parse_backslash(input: &[u8]) -> ParserResult<'_, ()> {
    match input.iter().next() {
        Some(b'\\') => split_at_matched(input, 1),
        _ => Err(ParserError::Generic),
    }
}

/// COLON = %x3A
pub(crate) fn parse_colon(input: &[u8]) -> ParserResult<'_, ()> {
    match input.iter().next() {
        Some(b':') => split_at_matched(input, 1),
        _ => Err(ParserError::Generic),
    }
}

/// SEMICOLON = %x3B
pub(crate) fn parse_semicolon(input: &[u8]) -> ParserResult<'_, ()> {
    match input.iter().next() {
        Some(b';') => split_at_matched(input, 1),
        _ => Err(ParserError::Generic),
    }
}

/// DOT = %x2E
pub(crate) fn parse_dot(input: &[u8]) -> ParserResult<'_, ()> {
    match input.iter().next() {
        Some(b'.') => split_at_matched(input, 1),
        _ => Err(ParserError::Generic),
    }
}

/// EQUALS = %x3D
pub(crate) fn parse_equals(input: &[u8]) -> ParserResult<'_, ()> {
    match input.iter().next() {
        Some(b'=') => split_at_matched(input, 1),
        _ => Err(ParserError::Generic),
    }
}

/// SAFE-CHAR = WSP / "!" / %x23-39 / %x3C-7E / NON-ASCII
fn parse_safe_char(input: &[u8]) -> ParserResult<'_, ()> {
    match input.iter().next() {
        Some(b' ' | b'\t' | b'!' | 0x23..=0x39 | 0x3C..=0x7E) => split_at_matched(input, 1),
        Some(c) if *c >= 0x80 => split_at_matched(input, utf8_width_from_first_byte(*c)?), // NON-ASCII
        _ => Err(ParserError::Generic),
    }
}

/// QSAFE-CHAR = WSP / "!" / %x23-7E / NON-ASCII
fn parse_qsafe_char(input: &[u8]) -> ParserResult<'_, ()> {
    match input.iter().next() {
        Some(b' ' | b'\t' | b'!' | 0x23..=0x7E) => split_at_matched(input, 1),
        Some(c) if *c >= 0x80 => split_at_matched(input, utf8_width_from_first_byte(*c)?), // NON-ASCII
        _ => Err(ParserError::Generic),
    }
}

/// VALUE-CHAR = WSP / VCHAR / NON-ASCII
fn parse_value_char(input: &[u8]) -> ParserResult<'_, ()> {
    match input.iter().next() {
        Some(b' ' | b'\t') => split_at_matched(input, 1), // WSP
        Some(0x21..=0x7E) => split_at_matched(input, 1),  // VCHAR
        Some(c) if *c >= 0x80 => split_at_matched(input, utf8_width_from_first_byte(*c)?), // NON-ASCII
        _ => Err(ParserError::Generic),
    }
}

/// Parse property value, without unescaping. Value can be empty.
pub(crate) fn parse_value(input: &[u8]) -> ParserResult<'_, ()> {
    let mut matched_len = 0;
    let mut local_input = input;
    while let Ok(out_char) = parse_value_char(local_input) {
        matched_len += out_char.matched().len();
        local_input = out_char.remaining();
    }
    let matched = &input[..matched_len];
    let remaining = &input[matched_len..];

    if !value_is_valid(matched) {
        return Err(ParserError::Generic);
    }

    Ok(ParserOutput::new(matched, remaining))
}

/// Check if a property value is valid according to RFC 6350.
fn value_is_valid(input: &[u8]) -> bool {
    let mut i = 0;
    while i < input.len() {
        let b1 = input[i];

        // Backslash characters need to be escaped OR be used to escape, everything else is OK
        // See "3.4. Property Value Escaping" of RFC6350
        if b1 != b'\\' {
            i += 1;
            continue;
        }

        // Lookahead
        i += 1;

        if i >= input.len() {
            // Backslash not followed by a char is invalid, it should be an escape char
            return false;
        }

        let b2 = input[i];
        // Per RFC, these are the only chars that can be escaped in property values
        if !b"\\nN,;".contains(&b2) {
            return false;
        }

        // Skip escaped char
        i += 1;
    }
    true
}

/// ALPHA = %x41-5A / %x61-7A
fn parse_alpha(input: &[u8]) -> ParserResult<'_, ()> {
    match input.iter().next() {
        Some(0x41..=0x5A) | Some(0x61..=0x7A) => split_at_matched(input, 1),
        _ => Err(ParserError::Generic),
    }
}

/// BIT = "0" / "1"
#[allow(unused)]
fn parse_bit(input: &[u8]) -> ParserResult<'_, ()> {
    match input.iter().next() {
        Some(b'0' | b'1') => split_at_matched(input, 1),
        _ => Err(ParserError::Generic),
    }
}

/// CHAR = %x01-7F
#[allow(unused)]
fn parse_char(input: &[u8]) -> ParserResult<'_, ()> {
    match input.iter().next() {
        Some(0x01..=0x7F) => split_at_matched(input, 1),
        _ => Err(ParserError::Generic),
    }
}

/// CR = %x0D
#[allow(unused)]
fn parse_cr(input: &[u8]) -> ParserResult<'_, ()> {
    match input.iter().next() {
        Some(b'\r') => split_at_matched(input, 1),
        _ => Err(ParserError::Generic),
    }
}

/// CTL = %x00-1F / %x7F
#[allow(unused)]
fn parse_ctl(input: &[u8]) -> ParserResult<'_, ()> {
    match input.iter().next() {
        Some(0x00..=0x1F) | Some(0x7F) => split_at_matched(input, 1),
        _ => Err(ParserError::Generic),
    }
}

/// DIGIT = %x30-39
fn parse_digit(input: &[u8]) -> ParserResult<'_, ()> {
    match input.iter().next() {
        Some(0x30..=0x39) => split_at_matched(input, 1),
        _ => Err(ParserError::Generic),
    }
}

/// DQUOTE = %x22
fn parse_dquote(input: &[u8]) -> ParserResult<'_, ()> {
    match input.iter().next() {
        Some(b'\"') => split_at_matched(input, 1),
        _ => Err(ParserError::Generic),
    }
}

/// HEXDIG = DIGIT / "A" / "B" / "C" / "D" / "E" / "F"
#[allow(unused)]
fn parse_hexdig(input: &[u8]) -> ParserResult<'_, ()> {
    match input.iter().next() {
        Some(0x30..=0x39) | Some(0x41..=0x46) | Some(0x61..=0x66) => split_at_matched(input, 1),
        _ => Err(ParserError::Generic),
    }
}

/// HTAB = %x09
#[allow(unused)]
fn parse_htab(input: &[u8]) -> ParserResult<'_, ()> {
    match input.iter().next() {
        Some(b'\t') => split_at_matched(input, 1),
        _ => Err(ParserError::Generic),
    }
}

/// LF = %x0A
#[allow(unused)]
fn parse_lf(input: &[u8]) -> ParserResult<'_, ()> {
    match input.iter().next() {
        Some(b'\n') => split_at_matched(input, 1),
        _ => Err(ParserError::Generic),
    }
}

/// SP = %x20
#[allow(unused)]
fn parse_sp(input: &[u8]) -> ParserResult<'_, ()> {
    match input.iter().next() {
        Some(&b' ') => split_at_matched(input, 1),
        _ => Err(ParserError::Generic),
    }
}

/// VCHAR = %x21-7E
#[allow(unused)]
fn parse_vchar(input: &[u8]) -> ParserResult<'_, ()> {
    match input.iter().next() {
        Some(0x21..=0x7E) => split_at_matched(input, 1),
        _ => Err(ParserError::Generic),
    }
}

/// WSP = SP / HTAB
#[allow(unused)]
pub(crate) fn parse_wsp(input: &[u8]) -> ParserResult<'_, ()> {
    match input.iter().next() {
        Some(b' ' | b'\t') => split_at_matched(input, 1),
        _ => Err(ParserError::Generic),
    }
}
