use crate::parser_internals::result::{ParserError, ParserOutput, ParserResult};

/// Helper function to split input into matched and remaining parts.
pub(crate) fn split_at_matched(input: &[u8], matched_len: usize) -> ParserResult<'_, ()> {
    assert!(
        matched_len <= input.len(),
        "shouldn't have matched_len > input.len()"
    );
    Ok(ParserOutput::new(
        &input[..matched_len],
        &input[matched_len..],
    ))
}

/// Check if a character at position is escaped
#[allow(unused)]
pub(crate) fn is_escaped(input: &[u8], position: usize) -> bool {
    if position == 0 {
        return false;
    }

    let mut backslash_count = 0;
    let mut i = position;

    while i > 0 {
        i -= 1;
        if input[i] == b'\\' {
            backslash_count += 1;
        } else {
            break;
        }
    }

    backslash_count % 2 == 1
}

/// Determine UTF-8 character width from the first byte. Returns an error when an invalid UTF8 byte is found.
pub(crate) fn utf8_width_from_first_byte(byte: u8) -> Result<usize, ParserError> {
    match byte {
        0x00..=0x7F => Ok(1),
        0xC2..=0xDF => Ok(2),
        0xE0..=0xEF => Ok(3),
        0xF0..=0xF4 => Ok(4),
        _ => Err(ParserError::InvalidUtf8),
    }
}
