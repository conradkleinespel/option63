use crate::Version;
use crate::param::{
    AltidParam, CalscaleParam, GeoParam, LabelParam, LanguageParam, MediatypeParam, OtherParam,
    ParamTrait, PidParam, PidParamValue, PrefParam, SortAsParam, TypeParam, TypeParamValue,
    TzParam, ValueParam,
};
use crate::parser_internals::ParseContext;
use crate::parser_internals::result::{ParserError, ParserOutput, ParserResult};
use crate::parser_internals::units::{
    parse_colon, parse_comma, parse_dot, parse_equals, parse_group, parse_param_name,
    parse_param_value, parse_property_name, parse_semicolon, parse_value, parse_wsp,
};
use crate::property_support::pref::{MAX_PREF, PropertyPref};
use crate::property_support::{Property, TryFromProperty};

#[macro_export]
macro_rules! v {
    ($($field:expr),* $(,)?) => {
        $crate::Value::new(vec![$($field),*])
    };
}

#[macro_export]
macro_rules! f {
    ($($val:expr),* $(,)?) => {
        $crate::Field::new(vec![$($val),*])
    };
}

#[macro_export]
macro_rules! fv {
    ($val:expr) => {
        $crate::FieldValue::from($val)
    };
}

/// A vCard.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VCard {
    content_lines: Vec<ContentLine>,
    version: Version,
}

impl VCard {
    fn new(content_lines: Vec<ContentLine>, version: Version) -> VCard {
        VCard {
            content_lines,
            version,
        }
    }

    /// Parse a vCard.
    ///
    /// Uses two-pass parsing:
    /// - Pass 1: Scan for VERSION property to determine vCard version
    /// - Pass 2: Parse all content lines with the determined version
    ///
    /// If VERSION is absent, defaults to VcardVersion::V21.
    /// For VcardVersion::V40, VERSION must be the second line (after BEGIN).
    pub fn parse(input: &[u8], strict: bool) -> ParserResult<'_, VCard> {
        let version = Self::extract_version_from_input(input, strict)?;
        let ctx = ParseContext::new(version, strict);

        let mut remaining = input;
        let mut vcard_begun = false;
        let mut total_matched_len = 0;
        let mut content_lines = Vec::new();

        while let Ok(out_line) = VCardFileLine::parse(remaining, strict) {
            let unfolded = out_line.output().unfold(strict).collect::<Vec<u8>>();
            let out_content_line = ContentLine::parse(&unfolded, ctx)?;

            let is_end = matches!(out_content_line.output().property, Property::End(_));

            if !vcard_begun {
                if !matches!(out_content_line.output().property, Property::Begin(_)) {
                    return Err(ParserError::MissingBegin);
                }
                vcard_begun = true;
            }

            total_matched_len += out_line.matched().len();
            remaining = out_line.remaining();
            content_lines.push(out_content_line.into_output());

            if vcard_begun && is_end {
                return Ok(ParserOutput::with_output(
                    &input[..total_matched_len],
                    remaining,
                    VCard::new(content_lines, version),
                ));
            }
        }

        if !vcard_begun {
            Err(ParserError::MissingBegin)
        } else {
            Err(ParserError::MissingEnd)
        }
    }

    fn extract_version_from_input(input: &[u8], strict: bool) -> Result<Version, ParserError> {
        let mut version_value: Option<Version> = None;
        let mut version_line_number = 0;
        let mut version_count = 0;

        let lines = Self::get_lines_until_next_end_property(input, strict);
        for (i, line_bytes) in lines.iter().enumerate() {
            if let Ok(out_name) = parse_property_name(line_bytes) {
                let name_upper = out_name.matched().to_ascii_uppercase();
                if name_upper != b"VERSION" {
                    continue;
                }

                version_count += 1;
                if version_count > 1 {
                    return Err(ParserError::MultipleVersion);
                }
                version_line_number = i;

                let mut current = out_name.remaining();

                // Skip parameters if any
                while let Ok(out_semicolon) = parse_semicolon(current) {
                    if let Ok(out_param_name) = parse_param_name(out_semicolon.remaining()) {
                        if let Ok(out_equals) = parse_equals(out_param_name.remaining()) {
                            let (matched_val, remaining_after_val, _) =
                                parse_param_value(out_equals.remaining());
                            if !matched_val.is_empty() {
                                current = remaining_after_val;
                                // Handle comma separated values
                                while let Ok(out_comma) = parse_comma(current) {
                                    let (next_matched, next_remaining, _) =
                                        parse_param_value(out_comma.remaining());
                                    if !next_matched.is_empty() {
                                        current = next_remaining;
                                    } else {
                                        break;
                                    }
                                }
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                if let Ok(out_colon) = parse_colon(current)
                    && let Ok(out_val) = parse_value(out_colon.remaining())
                {
                    version_value = match out_val.matched() {
                        b"2.1" => Some(Version::V21),
                        b"3.0" => Some(Version::V30),
                        b"4.0" => Some(Version::V40),
                        _ => None,
                    };
                }
            }
        }

        // v3.0 and v4.0 force the presence of VERSION, v2.1 doesn't
        let version = version_value.unwrap_or(Version::V21);
        if version == Version::V21 {
            return Err(ParserError::UnsupportedVersion(version));
        }

        // For vCard 4.0, VERSION must be the second line per RFC
        if version == Version::V40 && version_line_number != 1 {
            return Err(ParserError::VersionNotSecondLine);
        }

        Ok(version)
    }

    fn get_lines_until_next_end_property(input: &[u8], strict: bool) -> Vec<Vec<u8>> {
        let mut remaining = input;
        let mut lines_until_next_end_property: Vec<Vec<u8>> = Vec::new();

        while let Ok(out_line) = VCardFileLine::parse(remaining, strict) {
            let unfolded = out_line.output().unfold(strict).collect::<Vec<u8>>();
            if let Ok(out_name) = parse_property_name(&unfolded)
                && out_name.matched().eq_ignore_ascii_case(b"END")
            {
                break;
            }

            lines_until_next_end_property.push(unfolded);
            remaining = out_line.remaining();
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

/// Length of a line terminator at the start of `input`.
///
/// Strictly this is `\r\n` (2 bytes). In lax mode a lone `\n` (1 byte) is also
/// accepted as a terminator. Returns `None` when no terminator is present.
fn line_end_len(input: &[u8], strict: bool) -> Option<usize> {
    if input.len() >= 2 && &input[..2] == b"\r\n" {
        return Some(2);
    }

    if !strict && input.first() == Some(&b'\n') {
        return Some(1);
    }

    None
}

/// Length of a line fold at the start of `input`: a line terminator followed
/// by a single WSP character (line unfolding).
fn line_end_wsp_len(input: &[u8], strict: bool) -> Option<usize> {
    let end_len = line_end_len(input, strict)?;

    if let Ok(out_wsp) = parse_wsp(&input[end_len..]) {
        Some(end_len + out_wsp.matched().len())
    } else {
        None
    }
}

/// A vCard file line.
#[derive(Clone, Debug, Eq, PartialEq)]
struct VCardFileLine {
    input: Vec<u8>,
}

impl VCardFileLine {
    fn new(input: Vec<u8>) -> VCardFileLine {
        VCardFileLine { input }
    }

    /// Parse a vCard line, handling line folding as per RFC 2425/6350, Section 3.2.
    ///
    /// Line unfolding is handled by removing CRLF followed by a single WSP character.
    ///
    /// In non-strict mode, a lone LF is accepted as a line terminator, and a line
    /// without a trailing terminator at EOF is returned as the final line.
    fn parse(input: &[u8], strict: bool) -> ParserResult<'_, VCardFileLine> {
        let mut index = 0;

        while index < input.len() {
            if let Some(end_len) = line_end_len(&input[index..], strict) {
                match parse_wsp(&input[index + end_len..]) {
                    Ok(out_wsp) => {
                        index += end_len + out_wsp.matched().len();
                        continue;
                    }
                    Err(_) => {
                        index += end_len;
                        let line = VCardFileLine::new(input[..index].to_vec());

                        return Ok(ParserOutput::with_output(
                            &input[..index],
                            &input[index..],
                            line,
                        ));
                    }
                }
            }

            index += 1;
        }

        if !strict && index > 0 {
            let line = VCardFileLine::new(input.to_vec());
            return Ok(ParserOutput::with_output(input, &[], line));
        }

        Err(ParserError::Generic)
    }

    /// Create an iterator that unfolds the line.
    fn unfold(&self, strict: bool) -> VCardFileLineBytes {
        VCardFileLineBytes::new(self.input.clone(), strict)
    }
}

/// Iterator over bytes of a vCard line, performing unfolding.
#[derive(Clone, Debug, Eq, PartialEq)]
struct VCardFileLineBytes {
    input: Vec<u8>,
    index: usize,
    strict: bool,
}

impl VCardFileLineBytes {
    fn new(input: Vec<u8>, strict: bool) -> VCardFileLineBytes {
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

            if let Some(fold_len) = line_end_wsp_len(&self.input[self.index..], self.strict) {
                self.index += fold_len;
                continue;
            }

            if line_end_len(&self.input[self.index..], self.strict).is_some() {
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
    group: Option<Vec<u8>>,

    /// Property name and value.
    property: Property,
}

impl ContentLine {
    pub fn new(group: Option<Vec<u8>>, property: Property) -> ContentLine {
        ContentLine { group, property }
    }

    /// Parse a content line.
    ///
    /// input must be a line as returned by ContentLine::parse().unfold()
    /// contentline = [group "."] name *(";" param) ":" value CRLF
    fn parse(input: &[u8], ctx: ParseContext) -> ParserResult<'_, ContentLine> {
        let mut remaining = input;

        // Parse optional group: [group "."]
        let group = if let Ok(out_group) = parse_group(remaining) {
            if let Ok(out_dot) = parse_dot(out_group.remaining()) {
                remaining = out_dot.remaining();
                Some(out_group.matched().to_vec())
            } else {
                None
            }
        } else {
            None
        };

        // Parse required name
        let out_name = parse_property_name(remaining).map_err(|_| ParserError::PropertyName)?;
        remaining = out_name.remaining();

        // Parse zero or more parameters: *(";" param)
        let mut params = Vec::new();
        while let Ok(out_semicolon) = parse_semicolon(remaining) {
            let out_param = Param::parse(out_semicolon.remaining())?;
            remaining = out_param.remaining();
            params.push(out_param.into_output());
        }

        // Parse required ":"
        let out_colon = parse_colon(remaining).map_err(|_| ParserError::Colon)?;
        remaining = out_colon.remaining();

        // Parse required value
        let out_val = parse_value(remaining).map_err(|_| ParserError::PropertyValue)?;
        remaining = out_val.remaining();

        Ok(ParserOutput::with_output(
            &input[..input.len() - remaining.len()],
            remaining,
            ContentLine::new(
                group,
                Property::new(out_name.matched(), params, out_val.matched().to_vec(), ctx)?,
            ),
        ))
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

/// Fold a line to the maximum width per RFC 6350 Section 3.2
fn fold_line(input: &[u8], max_width: usize) -> Vec<u8> {
    const CONTINUATION: &[u8] = b"\r\n ";
    let mut result = vec![];
    let mut pos = 0;

    while pos < input.len() {
        let remaining = input.len() - pos;
        if remaining <= max_width {
            // No folding needed for remaining content
            result.extend(&input[pos..]);
            break;
        }

        // Find fold point at or before max_width
        // Must not split multi-byte UTF-8 characters
        let mut fold_at = max_width;
        while fold_at > 0 && (input[pos + fold_at] & 0b11000000) == 0b10000000 {
            // We're in the middle of a multi-byte UTF-8 char, move back
            fold_at -= 1;
        }

        if fold_at == 0 {
            // Entire max_width is a single multi-byte character, include it
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

#[non_exhaustive]
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum PropertyValueParseError {
    #[error("value fields must contain exactly one value, found {0}")]
    InvalidNumberFieldValues(usize),
    #[error("value must have one field, found {0}")]
    InvalidNumberFields(usize),
    #[error("unescape failed because of invalid input")]
    InvalidEscapeCharacters,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Value {
    content: Vec<Field>,
}

impl Value {
    pub fn new(content: Vec<Field>) -> Value {
        Value { content }
    }

    pub fn fields(&self) -> &[Field] {
        &self.content
    }
}

impl From<Vec<u8>> for Value {
    fn from(value: Vec<u8>) -> Self {
        v!(f!(fv!(value)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldValue {
    content: Vec<u8>,
    escape_field_separators: bool,
    escape_value_separators: bool,
}

impl FieldValue {
    pub fn new(content: Vec<u8>) -> FieldValue {
        FieldValue {
            content,
            escape_field_separators: true,
            escape_value_separators: true,
        }
    }
    pub fn raw(content: Vec<u8>) -> FieldValue {
        FieldValue {
            content,
            escape_field_separators: false,
            escape_value_separators: false,
        }
    }
    pub fn escape_field_separator(&self) -> bool {
        self.escape_field_separators
    }
    pub fn escape_value_separator(&self) -> bool {
        self.escape_value_separators
    }
    pub fn as_slice(&self) -> &[u8] {
        self.content.as_slice()
    }
    pub fn into_inner(self) -> Vec<u8> {
        self.content
    }
}

impl From<Vec<u8>> for FieldValue {
    fn from(value: Vec<u8>) -> Self {
        FieldValue::new(value)
    }
}

impl From<&[u8]> for FieldValue {
    fn from(value: &[u8]) -> Self {
        FieldValue::new(value.to_vec())
    }
}

impl From<String> for FieldValue {
    fn from(value: String) -> Self {
        FieldValue::new(value.into_bytes())
    }
}

impl From<&str> for FieldValue {
    fn from(value: &str) -> Self {
        FieldValue::new(value.as_bytes().to_vec())
    }
}

impl From<u8> for FieldValue {
    fn from(value: u8) -> Self {
        if value <= 127 {
            FieldValue::new(vec![value])
        } else {
            FieldValue::new(vec![b'?'])
        }
    }
}

impl From<&Vec<u8>> for FieldValue {
    fn from(value: &Vec<u8>) -> Self {
        FieldValue::new(value.clone())
    }
}

impl From<char> for FieldValue {
    fn from(value: char) -> Self {
        FieldValue::new(value.to_string().into_bytes())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    values: Vec<FieldValue>,
}

impl Field {
    pub fn new(values: Vec<FieldValue>) -> Field {
        Field { values }
    }
    pub fn values(&self) -> &[FieldValue] {
        self.values.as_slice()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Param {
    Calscale(CalscaleParam),
    Pref(PrefParam),
    Language(LanguageParam),
    Value(ValueParam),
    Altid(AltidParam),
    Label(LabelParam),
    Pid(PidParam),
    Type(TypeParam),
    Mediatype(MediatypeParam),
    SortAs(SortAsParam),
    Geo(GeoParam),
    Tz(TzParam),
    Other(OtherParam),
}

impl Param {
    /// Parse a parameter
    fn parse(input: &[u8]) -> ParserResult<'_, Param> {
        let mut matched_len = 0;
        let mut values = Vec::new();

        let out_name = parse_param_name(input).map_err(|_| ParserError::ParamName)?;
        matched_len += out_name.matched().len();
        let mut remaining = out_name.remaining();

        // Parse "="
        let out_equals = parse_equals(remaining).map_err(|_| ParserError::ParamEquals)?;
        matched_len += out_equals.matched().len();
        remaining = out_equals.remaining();

        // Parse first param-value
        let (first_value, new_remaining, raw_param_value) = parse_param_value(remaining);
        matched_len += first_value.len();
        values.push(raw_param_value.into_inner());
        remaining = new_remaining;

        let mut loop_remaining = remaining;
        // Parse additional param-values: *("," param-value)
        while let Ok(out_comma) = parse_comma(loop_remaining) {
            // Check if this comma is part of the current param or starts a new param
            // We need to look ahead to see if there's another param name after the comma
            matched_len += out_comma.matched().len();

            let (next_value, next_remaining, raw_param_value) =
                parse_param_value(out_comma.remaining());
            matched_len += next_value.len();
            values.push(raw_param_value.into_inner());
            loop_remaining = next_remaining;
        }

        assert!(
            matched_len <= input.len(),
            "shouldn't have matched_len > input.len()"
        );

        let param = match out_name.matched().to_ascii_uppercase().as_slice() {
            b"CALSCALE" => Param::Calscale(CalscaleParam::parse(values)?),
            b"PREF" => Param::Pref(PrefParam::parse(values)?),
            b"LANGUAGE" => Param::Language(LanguageParam::parse(values)?),
            b"VALUE" => Param::Value(ValueParam::parse(values)?),
            b"ALTID" => Param::Altid(AltidParam::parse(values)?),
            b"PID" => Param::Pid(PidParam::parse(values)?),
            b"TYPE" => Param::Type(TypeParam::parse(values)?),
            b"MEDIATYPE" => Param::Mediatype(MediatypeParam::parse(values)?),
            b"SORTAS" => Param::SortAs(SortAsParam::parse(values)?),
            b"GEO" => Param::Geo(GeoParam::parse(values)?),
            b"TZ" => Param::Tz(TzParam::parse(values)?),
            b"LABEL" => Param::Label(LabelParam::parse(values)?),
            _ => Param::Other(OtherParam::parse(out_name.matched().to_vec(), values)),
        };

        Ok(ParserOutput::with_output(
            &input[..matched_len],
            &input[matched_len..],
            param,
        ))
    }

    /// Get an iterator over the parameter name converted to uppercase.
    pub fn name(&self) -> &[u8] {
        match self {
            Param::Calscale(_) => b"CALSCALE",
            Param::Pref(_) => b"PREF",
            Param::Language(_) => b"LANGUAGE",
            Param::Value(_) => b"VALUE",
            Param::Altid(_) => b"ALTID",
            Param::Pid(_) => b"PID",
            Param::Type(_) => b"TYPE",
            Param::Mediatype(_) => b"MEDIATYPE",
            Param::SortAs(_) => b"SORTAS",
            Param::Geo(_) => b"GEO",
            Param::Tz(_) => b"TZ",
            Param::Label(_) => b"LABEL",
            Param::Other(other) => other.name(),
        }
    }

    pub fn values(&self) -> Vec<Vec<u8>> {
        match self {
            Param::Calscale(p) => match p {
                CalscaleParam::Gregorian => vec![b"gregorian".to_vec()],
                CalscaleParam::IanaToken(v) => vec![v.clone()],
                CalscaleParam::XName(v) => vec![v.clone()],
            },
            Param::Pref(p) => vec![p.value().to_string().into_bytes()],
            Param::Language(p) => match p {
                LanguageParam::Other(v) => vec![v.clone().into_bytes()],
            },
            Param::Value(p) => match p {
                ValueParam::Text => vec![b"text".to_vec()],
                ValueParam::Uri => vec![b"uri".to_vec()],
                ValueParam::Date => vec![b"date".to_vec()],
                ValueParam::Time => vec![b"time".to_vec()],
                ValueParam::DateTime => vec![b"date-time".to_vec()],
                ValueParam::DateAndOrTime => vec![b"date-and-or-time".to_vec()],
                ValueParam::Timestamp => vec![b"timestamp".to_vec()],
                ValueParam::Boolean => vec![b"boolean".to_vec()],
                ValueParam::Integer => vec![b"integer".to_vec()],
                ValueParam::Float => vec![b"float".to_vec()],
                ValueParam::UtcOffset => vec![b"utc-offset".to_vec()],
                ValueParam::LanguageTag => vec![b"language-tag".to_vec()],
                ValueParam::IanaToken(v) => vec![v.clone()],
                ValueParam::XName(v) => vec![v.clone()],
            },
            Param::Altid(p) => vec![p.value().to_vec()],
            Param::Pid(p) => p
                .value()
                .iter()
                .map(|v| match v {
                    PidParamValue::Single(n) => n.to_string().into_bytes(),
                    PidParamValue::Double(n1, n2) => format!("{}.{}", n1, n2).into_bytes(),
                })
                .collect(),
            Param::Type(p) => p
                .values()
                .iter()
                .map(|v| match v {
                    TypeParamValue::Work => b"work".to_vec(),
                    TypeParamValue::Home => b"home".to_vec(),
                    TypeParamValue::IanaToken(v) => v.to_ascii_lowercase().clone(),
                    TypeParamValue::XName(v) => v.to_ascii_lowercase().clone(),
                })
                .collect(),
            Param::Mediatype(_) => vec![], // FIXME
            Param::SortAs(p) => p.values().to_vec(),
            Param::Geo(p) => vec![p.uri().to_vec()],
            Param::Tz(p) => vec![p.value().to_vec()],
            Param::Label(p) => vec![p.value().to_vec()],
            Param::Other(p) => p.values().to_vec(),
        }
    }

    pub fn first_value(&self) -> Option<Vec<u8>> {
        self.values().into_iter().next()
    }
}
