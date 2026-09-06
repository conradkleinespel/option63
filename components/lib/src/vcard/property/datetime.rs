use crate::vcard::parser::{ParseError, R, RE};
use crate::vcard::property::param::Param;
use nom::Parser;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while_m_n};
use nom::character::complete::one_of;
use nom::combinator::opt;
use std::fmt::{Display, Formatter};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum DateAndOrTimeOrTextError {
    #[error("invalid date/time or text format")]
    InvalidFormat,
    #[error("invalid value parameter")]
    InvalidValueParam,
    #[error("unsupported version")]
    UnsupportedVersion,
}

pub(crate) fn parse_date_and_or_time_or_text_for_v40<'a>(
    value: &'a [u8],
    params: &[Param],
) -> RE<'a, DateAndOrTimeOrText, DateAndOrTimeOrTextError> {
    let value_type = params
        .iter()
        .find(|param| param.name() == b"VALUE")
        .and_then(|param| param.first_value())
        .unwrap_or(b"date-and-or-time".to_vec())
        .to_ascii_lowercase();

    match value_type.as_slice() {
        b"text" => {
            let s = String::from_utf8(value.to_vec())
                .map_err(|_| nom::Err::Error(DateAndOrTimeOrTextError::InvalidFormat))?;
            Ok((&value[s.len()..], DateAndOrTimeOrText::Text(s)))
        }
        b"date-and-or-time" => {
            let (rest, dt) = parse_date_and_or_time_for_v40(value)
                .map_err(|_| nom::Err::Error(DateAndOrTimeOrTextError::InvalidFormat))?;
            Ok((rest, dt))
        }
        _ => Err(nom::Err::Error(DateAndOrTimeOrTextError::InvalidValueParam)),
    }
}

// Subset of ISO8601
//
// Examples for "date":
// 1985-04-12
// 1996-08-05,1996-11-11
// 19850412
//
// Examples for "date-time":
// 1996-10-22T14:00:00Z
// 1996-08-11T12:34:56Z
// 19960811T123456Z
// 1996-10-22T14:00:00Z,1996-08-11T12:34:56Z
pub(crate) fn parse_date_or_date_time_for_v30<'a>(
    input: &'a [u8],
    _params: &[Param],
) -> RE<'a, DateAndOrTimeOrText, DateAndOrTimeOrTextError> {
    parse_date_time_iso8601_extended(input)
        .map(|(rest, dt)| (rest, DateAndOrTimeOrText::from(dt)))
        .or_else(|_| {
            parse_date_iso8601_extended(input)
                .map(|(rest, dt)| (rest, DateAndOrTimeOrText::from(dt)))
        })
        .or_else(|_| {
            parse_date_time_iso8601_basic(input)
                .map(|(rest, dt)| (rest, DateAndOrTimeOrText::from(dt)))
        })
        .or_else(|_| {
            parse_date_iso8601_basic(input).map(|(rest, dt)| (rest, DateAndOrTimeOrText::from(dt)))
        })
        // Some v3.0 vCards use this v4.0 syntax to be more compatible
        .or_else(|_| {
            parse_date_for_v40(input).map(|(rest, dt)| (rest, DateAndOrTimeOrText::from(dt)))
        })
        .map_err(|_| nom::Err::Error(DateAndOrTimeOrTextError::InvalidFormat))
}

/// Parse exactly `N` ASCII digits into a `T`.
fn digits_n<const N: usize, T>(input: &[u8]) -> R<'_, T>
where
    T: std::str::FromStr,
{
    let (rest, digits) = take_while_m_n(N, N, |b: u8| b.is_ascii_digit()).parse_complete(input)?;
    let s = std::str::from_utf8(digits).map_err(|_| nom::Err::Error(ParseError::Generic))?;
    let value = s
        .parse()
        .map_err(|_| nom::Err::Error(ParseError::Generic))?;
    Ok((rest, value))
}

// date-time / date / time-designator time
fn parse_date_and_or_time_for_v40(s: &[u8]) -> R<'_, DateAndOrTimeOrText> {
    if let Ok((rest, dt)) = parse_date_time_for_v40(s) {
        return Ok((rest, DateAndOrTimeOrText::DateTime(dt)));
    }
    if let Ok((rest, d)) = parse_date_for_v40(s) {
        return Ok((rest, DateAndOrTimeOrText::Date(d)));
    }
    let (rest, _) = parse_time_designator(s)?;
    let (rest, t) = parse_time_for_v40(rest)?;
    Ok((rest, DateAndOrTimeOrText::Time(t)))
}

// date-noreduc  time-designator time-notrunc
fn parse_date_time_for_v40(s: &[u8]) -> R<'_, DateTime> {
    let (rest, date) = parse_date_noreduc_for_v40(s)?;
    let (rest, _) = parse_time_designator(rest)?;
    let (rest, time) = parse_time_notrunc_for_v40(rest)?;
    Ok((rest, DateTime::new(date, time)))
}

fn parse_date_time_iso8601_basic(s: &[u8]) -> R<'_, DateTime> {
    let (rest, date) = parse_date_iso8601_basic(s)?;
    let (rest, _) = parse_time_designator(rest)?;
    let (rest, time) = parse_time_hour_minute_second(rest)?;
    let (rest, zone) = opt(parse_zone_iso8601_basic).parse_complete(rest)?;
    Ok((
        rest,
        DateTime::new(date, TimeWithZone::new(time, zone.flatten())),
    ))
}

// "T"
fn parse_time_designator(s: &[u8]) -> R<'_, ()> {
    tag("T").parse_complete(s).map(|(rest, _)| (rest, ()))
}

fn parse_date_time_iso8601_extended(s: &[u8]) -> R<'_, DateTime> {
    let (rest, date) = parse_date_iso8601_extended(s)?;
    let (rest, _) = parse_time_designator(rest)?;
    let (rest, time) = parse_time_iso8601_extended(rest)?;
    Ok((rest, DateTime::new(date, time)))
}

fn parse_date_iso8601_extended(s: &[u8]) -> R<'_, Date> {
    let (rest, year) = digits_n::<4, i32>(s)?;
    let (rest, _) = tag("-").parse_complete(rest)?;
    let (rest, month) = digits_n::<2, u32>(rest)?;
    let (rest, _) = tag("-").parse_complete(rest)?;
    let (rest, day) = digits_n::<2, u32>(rest)?;
    Ok((rest, Date::new(Some(year), Some(month), Some(day))))
}

fn parse_time_iso8601_extended(s: &[u8]) -> R<'_, TimeWithZone> {
    let (rest, hour) = digits_n::<2, u32>(s)?;
    let (rest, _) = tag(":").parse_complete(rest)?;
    let (rest, minute) = digits_n::<2, u32>(rest)?;
    let (rest, _) = tag(":").parse_complete(rest)?;
    let (rest, second) = digits_n::<2, u32>(rest)?;
    let (rest, zone) = parse_zone_iso8601_extended(rest)?;
    Ok((
        rest,
        TimeWithZone::new(Time::new(Some(hour), Some(minute), Some(second)), zone),
    ))
}

// year
fn parse_date_year_for_v40(s: &[u8]) -> R<'_, Date> {
    let (rest, year) = digits_n::<4, i32>(s)?;
    Ok((rest, Date::new(Some(year), None, None)))
}

// year "-" month
fn parse_date_year_month_for_v40(s: &[u8]) -> R<'_, Date> {
    let (rest, year) = digits_n::<4, i32>(s)?;
    let (rest, _) = tag("-").parse_complete(rest)?;
    let (rest, month) = digits_n::<2, u32>(rest)?;
    Ok((rest, Date::new(Some(year), Some(month), None)))
}

fn parse_date_iso8601_basic(s: &[u8]) -> R<'_, Date> {
    let (rest, year) = digits_n::<4, i32>(s)?;
    let (rest, month) = digits_n::<2, u32>(rest)?;
    let (rest, day) = digits_n::<2, u32>(rest)?;
    Ok((rest, Date::new(Some(year), Some(month), Some(day))))
}

// "--"     month
fn parse_date_month_for_v40(s: &[u8]) -> R<'_, Date> {
    let (rest, _) = tag("--").parse_complete(s)?;
    let (rest, month) = digits_n::<2, u32>(rest)?;
    Ok((rest, Date::new(None, Some(month), None)))
}

// "--"     month day
fn parse_date_month_day_for_v40(s: &[u8]) -> R<'_, Date> {
    let (rest, _) = tag("--").parse_complete(s)?;
    let (rest, month) = digits_n::<2, u32>(rest)?;
    let (rest, day) = digits_n::<2, u32>(rest)?;
    Ok((rest, Date::new(None, Some(month), Some(day))))
}

// "--"      "-"   day
fn parse_date_day_for_v40(s: &[u8]) -> R<'_, Date> {
    let (rest, _) = tag("--").parse_complete(s)?;
    let (rest, _) = tag("-").parse_complete(rest)?;
    let (rest, day) = digits_n::<2, u32>(rest)?;
    Ok((rest, Date::new(None, None, Some(day))))
}

// year    [month  day] / year "-" month / "--"     month [day] / "--"      "-"   day
fn parse_date_for_v40(s: &[u8]) -> R<'_, Date> {
    alt((
        parse_date_iso8601_basic,
        parse_date_year_month_for_v40,
        parse_date_year_for_v40,
        parse_date_month_day_for_v40,
        parse_date_month_for_v40,
        parse_date_day_for_v40,
    ))
    .parse_complete(s)
}

// year month day / "--" month  day / "--" "-" day
fn parse_date_noreduc_for_v40(s: &[u8]) -> R<'_, Date> {
    alt((
        parse_date_iso8601_basic,
        parse_date_month_day_for_v40,
        parse_date_day_for_v40,
    ))
    .parse_complete(s)
}

// 2DIGIT
fn parse_time_hour_for_v40(s: &[u8]) -> R<'_, Time> {
    let (rest, hour) = digits_n::<2, u32>(s)?;
    Ok((rest, Time::new(Some(hour), None, None)))
}

// 2DIGIT 2DIGIT
fn parse_time_hour_minute_for_v40(s: &[u8]) -> R<'_, Time> {
    let (rest, hour) = digits_n::<2, u32>(s)?;
    let (rest, minute) = digits_n::<2, u32>(rest)?;
    Ok((rest, Time::new(Some(hour), Some(minute), None)))
}

// hour minute second
fn parse_time_hour_minute_second(s: &[u8]) -> R<'_, Time> {
    let (rest, hour) = digits_n::<2, u32>(s)?;
    let (rest, minute) = digits_n::<2, u32>(rest)?;
    let (rest, second) = digits_n::<2, u32>(rest)?;
    Ok((rest, Time::new(Some(hour), Some(minute), Some(second))))
}

// "-"  minute second
fn parse_time_minute_second_for_v40(s: &[u8]) -> R<'_, Time> {
    let (rest, _) = tag("-").parse_complete(s)?;
    let (rest, minute) = digits_n::<2, u32>(rest)?;
    let (rest, second) = digits_n::<2, u32>(rest)?;
    Ok((rest, Time::new(None, Some(minute), Some(second))))
}

// "-" minute
fn parse_time_minute_for_v40(s: &[u8]) -> R<'_, Time> {
    let (rest, _) = tag("-").parse_complete(s)?;
    let (rest, minute) = digits_n::<2, u32>(rest)?;
    Ok((rest, Time::new(None, Some(minute), None)))
}

// "-"   "-"    second
fn parse_time_second_for_v40(s: &[u8]) -> R<'_, Time> {
    let (rest, _) = tag("--").parse_complete(s)?;
    let (rest, second) = digits_n::<2, u32>(rest)?;
    Ok((rest, Time::new(None, None, Some(second))))
}

// hour [minute [second]] [zone] / "-" minute [second] [zone] / "-" "-" second [zone]
fn parse_time_for_v40(s: &[u8]) -> R<'_, TimeWithZone> {
    let (rest, time) = alt((
        parse_time_hour_minute_second,
        parse_time_hour_minute_for_v40,
        parse_time_hour_for_v40,
        parse_time_minute_second_for_v40,
        parse_time_minute_for_v40,
        parse_time_second_for_v40,
    ))
    .parse_complete(s)?;
    let (rest, zone) = opt(parse_zone_iso8601_basic).parse_complete(rest)?;
    Ok((rest, TimeWithZone::new(time, zone.flatten())))
}

// hour [minute [second]] [zone]
fn parse_time_notrunc_for_v40(s: &[u8]) -> R<'_, TimeWithZone> {
    let (rest, time) = alt((
        parse_time_hour_minute_second,
        parse_time_hour_minute_for_v40,
        parse_time_hour_for_v40,
    ))
    .parse_complete(s)?;
    let (rest, zone) = opt(parse_zone_iso8601_basic).parse_complete(rest)?;
    Ok((rest, TimeWithZone::new(time, zone.flatten())))
}

// utc-designator = %x5A  ; uppercase "Z"
fn parse_zone_utc(s: &[u8]) -> R<'_, i32> {
    tag("Z").parse_complete(s).map(|(rest, _)| (rest, 0))
}

// sign digit digit ":" digit digit
fn parse_zone_sign_hh_mm(s: &[u8]) -> R<'_, i32> {
    let (rest, sign) = one_of("+-").parse_complete(s)?;
    let (rest, hour) = digits_n::<2, i32>(rest)?;
    let (rest, _) = tag(":").parse_complete(rest)?;
    let (rest, minute) = digits_n::<2, i32>(rest)?;
    let sign = if sign == '+' { 1 } else { -1 };
    Ok((rest, sign * (hour * 60 + minute)))
}

// Z / sign digit digit ":" digit digit
fn parse_zone_iso8601_extended(s: &[u8]) -> R<'_, Option<i32>> {
    if let Ok((rest, zone)) = parse_zone_utc(s) {
        return Ok((rest, Some(zone)));
    }
    if let Ok((rest, zone)) = parse_zone_sign_hh_mm(s) {
        return Ok((rest, Some(zone)));
    }
    Ok((s, None))
}

// zone = utc-designator / utc-offset
fn parse_zone_iso8601_basic(s: &[u8]) -> R<'_, Option<i32>> {
    if let Ok((rest, zone)) = parse_zone_utc(s) {
        return Ok((rest, Some(zone)));
    }
    parse_zone_utc_offset_iso8601_basic(s).map(|(rest, zone)| (rest, Some(zone)))
}

/// Parse exactly two ASCII digits into an `i32` (used for a zone hour/minute).
fn zone_two_digits(input: &[u8]) -> R<'_, i32> {
    digits_n::<2, i32>(input)
}

// ( "+" / "-" ) hour [minute]
fn parse_zone_utc_offset_iso8601_basic(s: &[u8]) -> R<'_, i32> {
    let (rest, sign) = one_of("+-").parse_complete(s)?;
    let (rest, hour) = digits_n::<2, i32>(rest)?;
    let (rest, minute) = opt(zone_two_digits).parse_complete(rest)?;
    let sign = if sign == '+' { 1 } else { -1 };
    Ok((rest, sign * (hour * 60 + minute.unwrap_or(0))))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DateAndOrTimeOrText {
    Text(String),
    Date(Date),
    Time(TimeWithZone),
    DateTime(DateTime),
}

impl From<String> for DateAndOrTimeOrText {
    fn from(s: String) -> Self {
        DateAndOrTimeOrText::Text(s)
    }
}

impl From<Date> for DateAndOrTimeOrText {
    fn from(d: Date) -> Self {
        DateAndOrTimeOrText::Date(d)
    }
}

impl From<TimeWithZone> for DateAndOrTimeOrText {
    fn from(t: TimeWithZone) -> Self {
        DateAndOrTimeOrText::Time(t)
    }
}

impl From<DateTime> for DateAndOrTimeOrText {
    fn from(dt: DateTime) -> Self {
        DateAndOrTimeOrText::DateTime(dt)
    }
}

impl Display for DateAndOrTimeOrText {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DateAndOrTimeOrText::Text(s) => write!(f, "{}", s),
            DateAndOrTimeOrText::Date(dt) => write!(f, "{}", dt),
            DateAndOrTimeOrText::Time(dt) => write!(f, "T{}", dt),
            DateAndOrTimeOrText::DateTime(dt) => write!(f, "{}", dt),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Date {
    pub(crate) year: Option<i32>,
    pub(crate) month: Option<u32>,
    pub(crate) day: Option<u32>,
}

impl Date {
    pub fn new(year: Option<i32>, month: Option<u32>, day: Option<u32>) -> Self {
        Self { year, month, day }
    }
}

impl Display for Date {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match (self.year, self.month, self.day) {
            (Some(y), Some(m), Some(d)) => write!(f, "{:04}{:02}{:02}", y, m, d),
            (Some(y), Some(m), None) => write!(f, "{:04}-{:02}", y, m),
            (Some(y), None, None) => write!(f, "{:04}", y),
            (None, Some(m), Some(d)) => write!(f, "--{:02}{:02}", m, d),
            (None, Some(m), None) => write!(f, "--{:02}", m),
            (None, None, Some(d)) => write!(f, "---{:02}", d),
            _ => Ok(()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimeWithZone {
    pub(crate) time: Time,
    pub(crate) zone: Option<i32>,
}

impl TimeWithZone {
    pub fn new(time: Time, zone: Option<i32>) -> Self {
        Self { time, zone }
    }
}

impl Display for TimeWithZone {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.time)?;
        if let Some(z) = self.zone {
            if z == 0 {
                return write!(f, "Z");
            } else {
                let sign = if z >= 0 { "+" } else { "-" };
                let z_abs = z.abs();
                let hours = z_abs / 60;
                let minutes = z_abs % 60;
                return write!(f, "{}{:02}{:02}", sign, hours, minutes);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Time {
    pub(crate) hour: Option<u32>,
    pub(crate) minute: Option<u32>,
    pub(crate) second: Option<u32>,
}

impl Time {
    pub fn new(hour: Option<u32>, minute: Option<u32>, second: Option<u32>) -> Self {
        Self {
            hour,
            minute,
            second,
        }
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(h) = self.hour {
            write!(f, "{:02}", h)?;
            if let Some(m) = self.minute {
                write!(f, "{:02}", m)?;
                if let Some(s) = self.second {
                    write!(f, "{:02}", s)?;
                }
            }
        } else if let Some(m) = self.minute {
            write!(f, "-{:02}", m)?;
            if let Some(s) = self.second {
                write!(f, "{:02}", s)?;
            }
        } else if let Some(s) = self.second {
            write!(f, "--{:02}", s)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DateTime {
    pub(crate) date: Date,
    pub(crate) time: TimeWithZone,
}

impl DateTime {
    pub fn new(date: Date, time: TimeWithZone) -> Self {
        Self { date, time }
    }
}

impl Display for DateTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}T{}", self.date, self.time)
    }
}
