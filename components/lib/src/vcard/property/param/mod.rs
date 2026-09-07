use crate::vcard::name;
use crate::vcard::parser::{ParseError, R};
use nom::Parser;
use nom::bytes::complete::{tag, take_while, take_while1};
use nom::multi::many0;
use nom::sequence::{delimited, preceded};

mod params;
pub use params::*;

pub trait ParamTrait: Sized {
    fn parse(values: Vec<Vec<u8>>) -> Result<Self, ParseError>;
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
            Param::Mediatype(p) => vec![p.raw().to_vec()],
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

    /// Parse a parameter with nom. Structural errors (missing name or equals)
    /// are reported as [`nom::Err::Failure`] so that they propagate through
    /// `many0`.
    pub(crate) fn parse(input: &[u8]) -> R<'_, Param> {
        let (rest, name_bytes) =
            name(input).map_err(|_| nom::Err::Failure(ParseError::ParamName))?;
        let (rest, _) = tag("=")
            .parse_complete(rest)
            .map_err(|_: nom::Err<ParseError>| nom::Err::Failure(ParseError::ParamEquals))?;

        let (rest, first_raw) = ParamValue::parse(rest)?;
        let mut values = vec![first_raw.into_inner()];
        let (rest, more_raw) = many0(preceded(tag(","), ParamValue::parse)).parse_complete(rest)?;
        values.extend(more_raw.into_iter().map(|pv| pv.into_inner()));

        let name_up = name_bytes.to_ascii_uppercase();
        let param = match name_up.as_slice() {
            b"CALSCALE" => CalscaleParam::parse(values).map(Param::Calscale),
            b"PREF" => PrefParam::parse(values).map(Param::Pref),
            b"LANGUAGE" => LanguageParam::parse(values).map(Param::Language),
            b"VALUE" => ValueParam::parse(values).map(Param::Value),
            b"ALTID" => AltidParam::parse(values).map(Param::Altid),
            b"PID" => PidParam::parse(values).map(Param::Pid),
            b"TYPE" => TypeParam::parse(values).map(Param::Type),
            b"MEDIATYPE" => MediatypeParam::parse(values).map(Param::Mediatype),
            b"SORTAS" => SortAsParam::parse(values).map(Param::SortAs),
            b"GEO" => GeoParam::parse(values).map(Param::Geo),
            b"TZ" => TzParam::parse(values).map(Param::Tz),
            b"LABEL" => LabelParam::parse(values).map(Param::Label),
            _ => Ok(Param::Other(OtherParam::parse(name_bytes.to_vec(), values))),
        };

        let param = param.map_err(nom::Err::Failure)?;

        Ok((rest, param))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParamValue {
    value: Vec<u8>,
    quoted: bool,
}

impl ParamValue {
    pub(crate) fn new(value: Vec<u8>, quoted: bool) -> Self {
        Self { value, quoted }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.value
    }

    /// Parse a parameter value.
    ///
    /// param-value = *SAFE-CHAR / DQUOTE *QSAFE-CHAR DQUOTE
    pub(crate) fn parse(input: &[u8]) -> R<'_, ParamValue> {
        // Unquoted: *SAFE-CHAR, stopping before a COMMA (SEMICOLON and COLON are
        // already excluded by SAFE-CHAR).
        let unquoted: R<'_, &[u8]> =
            take_while1(|b: u8| is_safe_char(b) && b != b',').parse_complete(input);
        if let Ok((rest, matched)) = unquoted {
            return Ok((rest, ParamValue::new(matched.to_vec(), false)));
        }

        // Quoted: DQUOTE *QSAFE-CHAR DQUOTE
        let quoted: R<'_, &[u8]> =
            delimited(tag("\""), take_while(|b: u8| is_qsafe_char(b)), tag("\""))
                .parse_complete(input);
        if let Ok((rest, inner)) = quoted {
            return Ok((rest, ParamValue::new(inner.to_vec(), true)));
        }

        // Empty value
        Ok((input, ParamValue::new(vec![], false)))
    }
}

/// SAFE-CHAR = WSP / "!" / %x23-39 / %x3C-7E / NON-ASCII
pub(crate) fn is_safe_char(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'!' | 0x23..=0x39 | 0x3C..=0x7E) || b >= 0x80
}

/// QSAFE-CHAR = WSP / "!" / %x23-7E / NON-ASCII
pub(crate) fn is_qsafe_char(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'!' | 0x23..=0x7E) || b >= 0x80
}
