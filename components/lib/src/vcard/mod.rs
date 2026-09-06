use crate::vcard::parser::{ParseContext, ParseError, R};
use crate::vcard::property::param::Param;
use crate::vcard::property::{Property, TryFromProperty, VersionError};
use nom::Parser;
use nom::branch::alt;
use nom::bytes::complete::take_while1;
use nom::bytes::tag;
use nom::character::complete::one_of;
use nom::combinator::{all_consuming, opt, recognize};
use nom::multi::many0;
use nom::sequence::{pair, preceded};
use property::{MAX_PREF, PropertyPref};

pub mod parser;
pub mod property;

/// Parse a property value, without unescaping. Value can be empty.
/// Succeeds only when every backslash in `input` is part of a valid escape
/// sequence (`\\nN,;`) and no lone trailing backslash remains.
pub(crate) fn value(input: &[u8]) -> R<'_, &[u8]> {
    all_consuming(recognize(many0(alt((
        // VCHAR = any visible US-ASCII character
        // VALUE-CHAR = WSP / VCHAR / NON-ASCII
        take_while1(|b| matches!(b, b' ' | b'\t' | 0x21..=0x5B | 0x5D..=0x7E) || b >= 0x80),
        recognize(preceded(
            nom::bytes::complete::tag("\\".as_bytes()),
            one_of("\\nN,;".as_bytes()),
        )),
    )))))
    .parse_complete(input)
    .map_err(|_: nom::Err<ParseError>| nom::Err::Failure(ParseError::PropertyValue))
}

/// property-name / param-name = iana-token / x-name
pub(crate) fn name(input: &[u8]) -> R<'_, &[u8]> {
    alt((x_name, iana_token)).parse_complete(input)
}

/// A line terminator: `\r\n`, or a lone `\n` in lax mode.
pub(crate) fn line_end(strict: bool) -> impl FnMut(&[u8]) -> R<'_, ()> {
    move |input: &[u8]| {
        if strict {
            recognize(tag("\r\n".as_bytes()))
                .map(|_| ())
                .parse_complete(input)
        } else {
            recognize(alt((tag("\r\n".as_bytes()), tag("\n".as_bytes()))))
                .map(|_| ())
                .parse_complete(input)
        }
    }
}

/// A line fold: a line terminator followed by a single WSP character (line
/// unfolding).
pub(crate) fn line_fold(strict: bool) -> impl FnMut(&[u8]) -> R<'_, ()> {
    move |input: &[u8]| {
        recognize(pair(line_end(strict), wsp))
            .map(|_| ())
            .parse_complete(input)
    }
}

/// group = 1*(ALPHA / DIGIT / "-")
pub(crate) fn group(input: &[u8]) -> R<'_, &[u8]> {
    take_while1(|b: u8| b.is_ascii_alphanumeric() || b == b'-').parse_complete(input)
}

/// iana-token = 1*(ALPHA / DIGIT / "-")
pub(crate) fn iana_token(input: &[u8]) -> R<'_, &[u8]> {
    take_while1(|b: u8| b.is_ascii_alphanumeric() || b == b'-').parse_complete(input)
}

/// x-name = "x-" 1*(ALPHA / DIGIT / "-")
pub(crate) fn x_name(input: &[u8]) -> R<'_, &[u8]> {
    recognize(preceded(nom::bytes::complete::tag("x-"), iana_token)).parse_complete(input)
}

fn wsp(input: &[u8]) -> R<'_, char> {
    one_of(" \t").parse_complete(input)
}

/// A single content byte that does not begin a line terminator.
pub(crate) fn line_plain_unit(strict: bool) -> impl FnMut(&[u8]) -> R<'_, ()> {
    move |input: &[u8]| {
        if input.is_empty() || line_end(strict).parse_complete(input).is_ok() {
            Err(nom::Err::Error(ParseError::Generic))
        } else {
            Ok((&input[1..], ()))
        }
    }
}

/// A line terminator: `\r\n`, or a lone `\n` in lax mode.
pub(crate) fn line_terminator_unit(strict: bool) -> impl FnMut(&[u8]) -> R<'_, ()> {
    move |input: &[u8]| line_end(strict).map(|_| ()).parse_complete(input)
}

/// Parse a content line with nom.
pub(crate) fn contentline(input: &[u8], ctx: ParseContext) -> R<'_, ContentLine> {
    // [group "."]
    let (rest, group) = opt(recognize(pair(group, tag(".")))).parse_complete(input)?;

    // name
    let (rest, name) = name(rest).map_err(|_| nom::Err::Failure(ParseError::PropertyName))?;

    // *(";" param)
    let (rest, params) = many0(preceded(tag(";"), Param::parse)).parse_complete(rest)?;

    // ":"
    let (rest, _) = tag(":")
        .parse_complete(rest)
        .map_err(|_: nom::Err<ParseError>| nom::Err::Failure(ParseError::Colon))?;

    // value
    let (rest, line_value) =
        value(rest).map_err(|_| nom::Err::Failure(ParseError::PropertyValue))?;

    let property =
        Property::new(name, params, line_value.to_vec(), ctx).map_err(nom::Err::Failure)?;

    let group = group.map(|g| g[..g.len() - 1].to_vec());
    Ok((rest, ContentLine { group, property }))
}

/// Fold a line to the maximum width per RFC 6350 Section 3.2
pub(crate) fn fold_line(input: &[u8], max_width: usize) -> Vec<u8> {
    const CONTINUATION: &[u8] = b"\r\n ";
    let mut result = vec![];
    let mut pos = 0;

    while pos < input.len() {
        let remaining = input.len() - pos;
        if remaining <= max_width {
            result.extend(&input[pos..]);
            break;
        }

        let mut fold_at = max_width;
        while fold_at > 0 && (input[pos + fold_at] & 0b11000000) == 0b10000000 {
            fold_at -= 1;
        }

        if fold_at == 0 {
            fold_at = 1;
            while pos + fold_at < input.len() && (input[pos + fold_at] & 0b11000000) == 0b10000000 {
                fold_at += 1;
            }
        }

        result.extend(&input[pos..pos + fold_at]);
        result.extend(CONTINUATION);
        pos += fold_at;
    }

    result
}

/// A vCard.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VCard {
    content_lines: Vec<ContentLine>,
    version: Version,
}

impl VCard {
    pub fn new(content_lines: Vec<ContentLine>, version: Version) -> VCard {
        VCard {
            content_lines,
            version,
        }
    }

    /// Parse a vCard.
    ///
    /// Returns the parsed [`VCard`] and the remaining input after its `END:VCARD`,
    /// allowing multiple concatenated vCards to be chained.
    ///
    /// Uses two-pass parsing:
    /// - Pass 1: Scan for VERSION property to determine vCard version
    /// - Pass 2: Parse all content lines with the determined version
    ///
    /// If VERSION is absent, defaults to VcardVersion::V21.
    /// For VcardVersion::V40, VERSION must be the second line (after BEGIN).
    pub fn parse(input: &[u8], strict: bool) -> Result<(&[u8], VCard), ParseError> {
        let version = Self::extract_version_from_input(input, strict)?;
        let ctx = ParseContext::new(version, strict);

        let mut remaining = input;
        let mut vcard_begun = false;
        let mut content_lines = Vec::new();

        while let Ok((rest, out_line)) = VCardFileLine::parse(remaining, strict) {
            let unfolded = out_line.unfold(strict).collect::<Vec<u8>>();
            let out_content_line = ContentLine::parse(&unfolded, ctx)?;

            let is_end = matches!(out_content_line.property(), Property::End(_));

            if !vcard_begun {
                if !matches!(out_content_line.property(), Property::Begin(_)) {
                    return Err(ParseError::MissingBegin);
                }
                vcard_begun = true;
            }

            remaining = rest;
            content_lines.push(out_content_line);

            if vcard_begun && is_end {
                return Ok((remaining, VCard::new(content_lines, version)));
            }
        }

        if !vcard_begun {
            Err(ParseError::MissingBegin)
        } else {
            Err(ParseError::MissingEnd)
        }
    }

    fn extract_version_from_input(input: &[u8], strict: bool) -> Result<Version, ParseError> {
        let mut version_str: Option<String> = None;
        let mut version_line_number = 0;
        let mut version_count = 0;

        let lines = Self::get_lines_until_next_end_property(input, strict);
        for (i, line_bytes) in lines.iter().enumerate() {
            let cl = match contentline(line_bytes, ParseContext::v40_lax()) {
                Err(err) => {
                    match err {
                        nom::Err::Error(e) | nom::Err::Failure(e) => match e {
                            ParseError::InvalidVersion(_) => return Err(e),
                            _ => continue,
                        },
                        nom::Err::Incomplete(_) => {
                            unreachable!("using nom streaming instead of complete mode")
                        }
                    };
                }
                Ok((_, cl)) => cl,
            };
            if !cl.property().name().eq_ignore_ascii_case(b"VERSION") {
                continue;
            }

            version_count += 1;
            if version_count > 1 {
                return Err(ParseError::MultipleVersion);
            }
            version_line_number = i;
            version_str = String::from_utf8(cl.property().value_to_vcard_vec().to_vec()).ok();
        }

        match version_str.as_deref() {
            Some("2.1") => Err(ParseError::InvalidVersion(
                VersionError::UnsupportedVersion("2.1".to_string()),
            )),
            Some("3.0") => Ok(Version::V30),
            Some("4.0") => {
                if version_line_number != 1 {
                    // TODO: allow this in lax mode
                    Err(ParseError::VersionNotSecondLine)
                } else {
                    Ok(Version::V40)
                }
            }
            Some(other) => Err(ParseError::InvalidVersion(
                VersionError::UnsupportedVersion(other.to_string()),
            )),
            None => Err(ParseError::InvalidVersion(
                VersionError::UnsupportedVersion("2.1".to_string()),
            )),
        }
    }

    fn get_lines_until_next_end_property(input: &[u8], strict: bool) -> Vec<Vec<u8>> {
        let mut remaining = input;
        let mut lines_until_next_end_property: Vec<Vec<u8>> = Vec::new();

        while let Ok((rest, out_line)) = VCardFileLine::parse(remaining, strict) {
            let unfolded = out_line.unfold(strict).collect::<Vec<u8>>();
            if let Ok((_, cl)) = contentline(&unfolded, ParseContext::v40_lax())
                && cl.property().name().eq_ignore_ascii_case(b"END")
            {
                break;
            }

            lines_until_next_end_property.push(unfolded);
            remaining = rest;
        }

        lines_until_next_end_property
    }

    pub fn get_preferred<T: TryFromProperty + PropertyPref>(&self) -> Option<&T> {
        let mut props = self
            .content_lines
            .iter()
            .filter_map(|cl| T::try_from_property(&cl.property))
            .collect::<Vec<&T>>();

        props.sort_by(|a, b| {
            a.pref()
                .unwrap_or(MAX_PREF)
                .cmp(&b.pref().unwrap_or(MAX_PREF))
        });

        props.first().copied()
    }

    pub fn version(&self) -> Version {
        self.version
    }

    pub fn content_lines(&self) -> &[ContentLine] {
        &self.content_lines
    }
}

/// A vCard file line.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VCardFileLine {
    input: Vec<u8>,
}

impl VCardFileLine {
    pub fn new(input: Vec<u8>) -> VCardFileLine {
        VCardFileLine { input }
    }

    /// Parse a vCard line, handling line folding as per RFC 2425/6350, Section 3.2.
    ///
    /// Returns the line's raw bytes and the remaining input after it.
    ///
    /// In non-strict mode, a lone LF is accepted as a line terminator, and a line
    /// without a trailing terminator at EOF is returned as the final line.
    pub fn parse(input: &[u8], strict: bool) -> Result<(&[u8], VCardFileLine), ParseError> {
        // A line is consumed content (plain bytes and folds) terminated by the
        // first line end that is not part of a fold (Section 3.2 unfolding).
        let parsed: R<'_, &[u8]> = recognize(pair(
            many0(alt((line_fold(strict), line_plain_unit(strict)))),
            line_terminator_unit(strict),
        ))
        .parse_complete(input);
        match parsed {
            Ok((rest, line)) => Ok((rest, VCardFileLine::new(line.to_vec()))),
            // Lax mode: a final line without a trailing terminator at EOF is
            // still a valid line.
            Err(_) if !strict && !input.is_empty() => Ok((&[], VCardFileLine::new(input.to_vec()))),
            Err(_) => Err(ParseError::Generic),
        }
    }

    /// Create an iterator that unfolds the line.
    pub fn unfold(&self, strict: bool) -> VCardFileLineBytes {
        VCardFileLineBytes::new(self.input.clone(), strict)
    }
}

/// Iterator over bytes of a vCard line, performing unfolding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VCardFileLineBytes {
    input: Vec<u8>,
    index: usize,
    strict: bool,
}

impl VCardFileLineBytes {
    pub(crate) fn new(input: Vec<u8>, strict: bool) -> VCardFileLineBytes {
        VCardFileLineBytes {
            input,
            index: 0,
            strict,
        }
    }
}

impl Iterator for VCardFileLineBytes {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.index >= self.input.len() {
                return None;
            }

            let remaining = &self.input[self.index..];

            if let Ok((rest, _)) = line_fold(self.strict).parse_complete(remaining) {
                self.index = self.input.len() - rest.len();
                continue;
            }

            if line_end(self.strict).parse_complete(remaining).is_ok() {
                return None;
            }

            let byte = self.input[self.index];
            self.index += 1;

            return Some(byte);
        }
    }
}

/// Represents a parsed vCard content line.
///
/// From RFC 6350: contentline = [group "."] name *(";" param) ":" value CRLF.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentLine {
    /// Optional group identifier (case-insensitive).
    /// group = 1*(ALPHA / DIGIT / "-")
    pub(crate) group: Option<Vec<u8>>,

    /// Property name and value.
    pub(crate) property: Property,
}

impl ContentLine {
    pub fn new(group: Option<Vec<u8>>, property: Property) -> ContentLine {
        ContentLine { group, property }
    }

    /// Parse a content line.
    ///
    /// input must be an unfolded line.
    /// contentline = [group "."] name *(";" param) ":" value CRLF
    pub fn parse(input: &[u8], ctx: ParseContext) -> Result<ContentLine, ParseError> {
        contentline(input, ctx)
            .map(|(_, content_line)| content_line)
            .map_err(|e| match e {
                nom::Err::Error(e) | nom::Err::Failure(e) => e,
                nom::Err::Incomplete(_) => ParseError::Generic,
            })
    }

    pub fn property(&self) -> &Property {
        &self.property
    }

    pub fn params(&self) -> &[Param] {
        self.property.params()
    }

    pub fn to_vcard_vec(&self) -> Vec<u8> {
        let mut out = vec![];
        if let Some(ref group) = self.group {
            out.extend(group.as_slice());
            out.push(b'.');
        }
        out.extend(self.property.to_vcard_vec());

        fold_line(out.as_slice(), 75)
    }
}

/// vCard version enumeration.
///
/// This enum is non-exhaustive to allow future versions to be added.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Version {
    /// vCard 2.1 (default if VERSION property is absent)
    V21,
    /// vCard 3.0 (RFC 2426)
    V30,
    /// vCard 4.0 (RFC 6350)
    V40,
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Version::V21 => write!(f, "2.1"),
            Version::V30 => write!(f, "3.0"),
            Version::V40 => write!(f, "4.0"),
        }
    }
}
