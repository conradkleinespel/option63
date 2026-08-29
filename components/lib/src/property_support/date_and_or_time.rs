use crate::parser_internals::model::Param;
use crate::parser_internals::result::{ParserError, ParserOutput, ParserResult};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DateAndOrTimeOrText {
    Text(String),
    DateAndOrTime(DateAndOrTime),
}

impl Display for DateAndOrTimeOrText {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DateAndOrTimeOrText::Text(s) => write!(f, "{}", s),
            DateAndOrTimeOrText::DateAndOrTime(dt) => write!(f, "{}", dt),
        }
    }
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum DateAndOrTimeOrTextError {
    #[error("invalid date/time or text format")]
    InvalidFormat,
    #[error("invalid value parameter")]
    InvalidValueParam,
    #[error("unsupported version")]
    UnsupportedVersion,
}

pub(crate) fn parse_date_and_or_time_or_text_for_v40(
    value: Vec<u8>,
    params: &[Param],
) -> Result<DateAndOrTimeOrText, DateAndOrTimeOrTextError> {
    let value_type = params
        .iter()
        .find(|param| param.name() == b"VALUE")
        .and_then(|param| param.first_value())
        .unwrap_or(b"date-and-or-time".to_vec())
        .to_ascii_lowercase();

    match value_type.as_slice() {
        b"text" => {
            let s =
                String::from_utf8(value).map_err(|_| DateAndOrTimeOrTextError::InvalidFormat)?;
            Ok(DateAndOrTimeOrText::Text(s))
        }
        b"date-and-or-time" => {
            let dt = parse_date_and_or_time_for_v40(&value)
                .map_err(|_| DateAndOrTimeOrTextError::InvalidFormat)?;
            Ok(DateAndOrTimeOrText::DateAndOrTime(dt.into_output()))
        }
        _ => Err(DateAndOrTimeOrTextError::InvalidValueParam),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DateAndOrTime {
    Date(Date),
    Time(TimeWithZone),
    DateTime(DateTime),
}

impl Display for DateAndOrTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DateAndOrTime::Date(d) => write!(f, "{}", d),
            DateAndOrTime::Time(t) => write!(f, "T{}", t),
            DateAndOrTime::DateTime(dt) => write!(f, "{}", dt),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Date {
    year: Option<i32>,
    month: Option<u32>,
    day: Option<u32>,
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
    time: Time,
    zone: Option<i32>,
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
    hour: Option<u32>,
    minute: Option<u32>,
    second: Option<u32>,
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
    date: Date,
    time: TimeWithZone,
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

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum DateAndOrTimeError {
    #[error("invalid date/time format")]
    InvalidFormat,
}

// date-time / date / time-designator time
fn parse_date_and_or_time_for_v40(s: &[u8]) -> ParserResult<'_, DateAndOrTime> {
    parse_date_time_for_v40(s)
        .map(|out| {
            ParserOutput::with_output(
                out.matched(),
                out.remaining(),
                DateAndOrTime::DateTime(out.into_output()),
            )
        })
        .or_else(|_| {
            parse_date_for_v40(s).map(|out| {
                ParserOutput::with_output(
                    out.matched(),
                    out.remaining(),
                    DateAndOrTime::Date(out.into_output()),
                )
            })
        })
        .or_else(|_| {
            let res_t = parse_time_designator(s)?;
            let res_time = parse_time_for_v40(res_t.remaining())?;
            Ok(ParserOutput::with_output(
                &s[..res_t.matched().len() + res_time.matched().len()],
                res_time.remaining(),
                DateAndOrTime::Time(res_time.into_output()),
            ))
        })
}

// date-noreduc  time-designator time-notrunc
fn parse_date_time_for_v40(s: &[u8]) -> ParserResult<'_, DateTime> {
    let res_date = parse_date_noreduc_for_v40(s)?;
    let res_t = parse_time_designator(res_date.remaining())?;
    let res_time = parse_time_notrunc_for_v40(res_t.remaining())?;

    Ok(ParserOutput::with_output(
        &s[..res_date.matched().len() + res_t.matched().len() + res_time.matched().len()],
        res_time.remaining(),
        DateTime::new(res_date.into_output(), res_time.into_output()),
    ))
}

fn parse_date_time_iso8601_basic(s: &[u8]) -> ParserResult<'_, DateTime> {
    let res_date = parse_date_iso8601_basic(s)?;
    let res_t = parse_time_designator(res_date.remaining())?;
    let res_time = parse_time_hour_minute_second(res_t.remaining())?;
    let res_zone = parse_zone_iso8601_basic(res_time.remaining())
        .unwrap_or(ParserOutput::with_output(&[], res_time.remaining(), None));

    Ok(ParserOutput::with_output(
        &s[..res_date.matched().len()
            + res_t.matched().len()
            + res_time.matched().len()
            + res_zone.matched().len()],
        res_zone.remaining(),
        DateTime::new(
            res_date.into_output(),
            TimeWithZone::new(res_time.into_output(), res_zone.into_output()),
        ),
    ))
}

// "T"
fn parse_time_designator(s: &[u8]) -> ParserResult<'_, ()> {
    if s.starts_with(b"T") {
        return Ok(ParserOutput::new(&s[..1], &s[1..]));
    }
    Err(ParserError::Generic)
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
pub(crate) fn parse_date_or_date_time_for_v30(
    input: &[u8],
    _params: &[Param],
) -> Result<DateAndOrTimeOrText, DateAndOrTimeError> {
    parse_date_time_iso8601_extended(input)
        .map(|res| DateAndOrTimeOrText::DateAndOrTime(DateAndOrTime::DateTime(res.into_output())))
        .or_else(|_| {
            parse_date_iso8601_extended(input).map(|res| {
                DateAndOrTimeOrText::DateAndOrTime(DateAndOrTime::Date(res.into_output()))
            })
        })
        .or_else(|_| {
            parse_date_time_iso8601_basic(input).map(|res| {
                DateAndOrTimeOrText::DateAndOrTime(DateAndOrTime::DateTime(res.into_output()))
            })
        })
        .or_else(|_| {
            parse_date_iso8601_basic(input).map(|res| {
                DateAndOrTimeOrText::DateAndOrTime(DateAndOrTime::Date(res.into_output()))
            })
        })
        .or_else(|_| {
            // Some v3.0 vCards use this v4.0 syntax to be more compatible
            parse_date_for_v40(input).map(|res| {
                DateAndOrTimeOrText::DateAndOrTime(DateAndOrTime::Date(res.into_output()))
            })
        })
        .map_err(|_| DateAndOrTimeError::InvalidFormat)
}

fn parse_date_time_iso8601_extended(s: &[u8]) -> ParserResult<'_, DateTime> {
    let res_date = parse_date_iso8601_extended(s)?;
    let res_t = parse_time_designator(res_date.remaining())?;
    let res_time = parse_time_iso8601_extended(res_t.remaining())?;

    Ok(ParserOutput::with_output(
        &s[..res_date.matched().len() + res_t.matched().len() + res_time.matched().len()],
        res_time.remaining(),
        DateTime::new(res_date.into_output(), res_time.into_output()),
    ))
}

fn parse_date_iso8601_extended(s: &[u8]) -> ParserResult<'_, Date> {
    if s.len() >= 10
        && &s[4..5] == b"-"
        && &s[7..8] == b"-"
        && s[..4].iter().all(|b| b.is_ascii_digit())
        && s[5..7].iter().all(|b| b.is_ascii_digit())
        && s[8..10].iter().all(|b| b.is_ascii_digit())
    {
        let year = std::str::from_utf8(&s[..4])
            .map_err(|_| ParserError::Generic)?
            .parse()
            .map_err(|_| ParserError::Generic)?;
        let month = std::str::from_utf8(&s[5..7])
            .map_err(|_| ParserError::Generic)?
            .parse()
            .map_err(|_| ParserError::Generic)?;
        let day = std::str::from_utf8(&s[8..10])
            .map_err(|_| ParserError::Generic)?
            .parse()
            .map_err(|_| ParserError::Generic)?;
        return Ok(ParserOutput::with_output(
            &s[..10],
            &s[10..],
            Date::new(Some(year), Some(month), Some(day)),
        ));
    }
    Err(ParserError::Generic)
}

fn parse_time_iso8601_extended(s: &[u8]) -> ParserResult<'_, TimeWithZone> {
    if s.len() >= 8
        && &s[2..3] == b":"
        && &s[5..6] == b":"
        && s[..2].iter().all(|b| b.is_ascii_digit())
        && s[3..5].iter().all(|b| b.is_ascii_digit())
        && s[6..8].iter().all(|b| b.is_ascii_digit())
    {
        let hour = std::str::from_utf8(&s[..2])
            .map_err(|_| ParserError::Generic)?
            .parse()
            .map_err(|_| ParserError::Generic)?;
        let minute = std::str::from_utf8(&s[3..5])
            .map_err(|_| ParserError::Generic)?
            .parse()
            .map_err(|_| ParserError::Generic)?;
        let second = std::str::from_utf8(&s[6..8])
            .map_err(|_| ParserError::Generic)?
            .parse()
            .map_err(|_| ParserError::Generic)?;
        let rem = &s[8..];
        let zone_res = parse_zone_iso8601_extended(rem)?;
        return Ok(ParserOutput::with_output(
            &s[..8 + zone_res.matched().len()],
            zone_res.remaining(),
            TimeWithZone::new(
                Time::new(Some(hour), Some(minute), Some(second)),
                zone_res.into_output(),
            ),
        ));
    }
    Err(ParserError::Generic)
}

// year
fn parse_date_year_for_v40(s: &[u8]) -> ParserResult<'_, Date> {
    if s.len() >= 4 && s[..4].iter().all(|b| b.is_ascii_digit()) {
        let year = std::str::from_utf8(&s[..4])
            .map_err(|_| ParserError::Generic)?
            .parse()
            .map_err(|_| ParserError::Generic)?;
        return Ok(ParserOutput::with_output(
            &s[..4],
            &s[4..],
            Date::new(Some(year), None, None),
        ));
    }
    Err(ParserError::Generic)
}

// year "-" month
fn parse_date_year_month_for_v40(s: &[u8]) -> ParserResult<'_, Date> {
    if s.len() >= 7
        && &s[4..5] == b"-"
        && s[..4].iter().all(|b| b.is_ascii_digit())
        && s[5..7].iter().all(|b| b.is_ascii_digit())
    {
        let year = std::str::from_utf8(&s[..4])
            .map_err(|_| ParserError::Generic)?
            .parse()
            .map_err(|_| ParserError::Generic)?;
        let month = std::str::from_utf8(&s[5..7])
            .map_err(|_| ParserError::Generic)?
            .parse()
            .map_err(|_| ParserError::Generic)?;
        return Ok(ParserOutput::with_output(
            &s[..7],
            &s[7..],
            Date::new(Some(year), Some(month), None),
        ));
    }
    Err(ParserError::Generic)
}

fn parse_date_iso8601_basic(s: &[u8]) -> ParserResult<'_, Date> {
    if s.len() >= 8 && s[..8].iter().all(|b| b.is_ascii_digit()) {
        let year = std::str::from_utf8(&s[..4])
            .map_err(|_| ParserError::Generic)?
            .parse()
            .map_err(|_| ParserError::Generic)?;
        let month = std::str::from_utf8(&s[4..6])
            .map_err(|_| ParserError::Generic)?
            .parse()
            .map_err(|_| ParserError::Generic)?;
        let day = std::str::from_utf8(&s[6..8])
            .map_err(|_| ParserError::Generic)?
            .parse()
            .map_err(|_| ParserError::Generic)?;
        return Ok(ParserOutput::with_output(
            &s[..8],
            &s[8..],
            Date::new(Some(year), Some(month), Some(day)),
        ));
    }
    Err(ParserError::Generic)
}

// "--"     month
fn parse_date_month_for_v40(s: &[u8]) -> ParserResult<'_, Date> {
    if s.starts_with(b"--") && s.len() >= 4 && s[2..4].iter().all(|b| b.is_ascii_digit()) {
        let month = std::str::from_utf8(&s[2..4])
            .map_err(|_| ParserError::Generic)?
            .parse()
            .map_err(|_| ParserError::Generic)?;
        return Ok(ParserOutput::with_output(
            &s[..4],
            &s[4..],
            Date::new(None, Some(month), None),
        ));
    }
    Err(ParserError::Generic)
}

// "--"     month day
fn parse_date_month_day_for_v40(s: &[u8]) -> ParserResult<'_, Date> {
    if s.starts_with(b"--") && s.len() >= 6 && s[2..6].iter().all(|b| b.is_ascii_digit()) {
        let month = std::str::from_utf8(&s[2..4])
            .map_err(|_| ParserError::Generic)?
            .parse()
            .map_err(|_| ParserError::Generic)?;
        let day = std::str::from_utf8(&s[4..6])
            .map_err(|_| ParserError::Generic)?
            .parse()
            .map_err(|_| ParserError::Generic)?;
        return Ok(ParserOutput::with_output(
            &s[..6],
            &s[6..],
            Date::new(None, Some(month), Some(day)),
        ));
    }
    Err(ParserError::Generic)
}

// "--"      "-"   day
fn parse_date_day_for_v40(s: &[u8]) -> ParserResult<'_, Date> {
    if s.starts_with(b"--")
        && s.len() >= 5
        && &s[2..3] == b"-"
        && s[3..5].iter().all(|b| b.is_ascii_digit())
    {
        let day = std::str::from_utf8(&s[3..5])
            .map_err(|_| ParserError::Generic)?
            .parse()
            .map_err(|_| ParserError::Generic)?;
        return Ok(ParserOutput::with_output(
            &s[..5],
            &s[5..],
            Date::new(None, None, Some(day)),
        ));
    }
    Err(ParserError::Generic)
}

// year    [month  day] / year "-" month / "--"     month [day] / "--"      "-"   day
fn parse_date_for_v40(s: &[u8]) -> ParserResult<'_, Date> {
    parse_date_iso8601_basic(s)
        .or_else(|_| parse_date_year_month_for_v40(s))
        .or_else(|_| parse_date_year_for_v40(s))
        .or_else(|_| parse_date_month_day_for_v40(s))
        .or_else(|_| parse_date_month_for_v40(s))
        .or_else(|_| parse_date_day_for_v40(s))
}

// year month day / "--" month  day / "--" "-" day
fn parse_date_noreduc_for_v40(s: &[u8]) -> ParserResult<'_, Date> {
    parse_date_iso8601_basic(s)
        .or_else(|_| parse_date_month_day_for_v40(s))
        .or_else(|_| parse_date_day_for_v40(s))
}

// 2DIGIT
fn parse_time_hour_for_v40(s: &[u8]) -> ParserResult<'_, Time> {
    if s.len() >= 2 && s[..2].iter().all(|b| b.is_ascii_digit()) {
        let hour = std::str::from_utf8(&s[..2])
            .map_err(|_| ParserError::Generic)?
            .parse()
            .map_err(|_| ParserError::Generic)?;
        return Ok(ParserOutput::with_output(
            &s[..2],
            &s[2..],
            Time::new(Some(hour), None, None),
        ));
    }
    Err(ParserError::Generic)
}

// 2DIGIT 2DIGIT
fn parse_time_hour_minute_for_v40(s: &[u8]) -> ParserResult<'_, Time> {
    if s.len() >= 4 && s[..4].iter().all(|b| b.is_ascii_digit()) {
        let hour = std::str::from_utf8(&s[..2])
            .map_err(|_| ParserError::Generic)?
            .parse()
            .map_err(|_| ParserError::Generic)?;
        let minute = std::str::from_utf8(&s[2..4])
            .map_err(|_| ParserError::Generic)?
            .parse()
            .map_err(|_| ParserError::Generic)?;
        return Ok(ParserOutput::with_output(
            &s[..4],
            &s[4..],
            Time::new(Some(hour), Some(minute), None),
        ));
    }
    Err(ParserError::Generic)
}

// hour minute second
fn parse_time_hour_minute_second(s: &[u8]) -> ParserResult<'_, Time> {
    if s.len() >= 6 && s[..6].iter().all(|b| b.is_ascii_digit()) {
        let hour = std::str::from_utf8(&s[..2])
            .map_err(|_| ParserError::Generic)?
            .parse()
            .map_err(|_| ParserError::Generic)?;
        let minute = std::str::from_utf8(&s[2..4])
            .map_err(|_| ParserError::Generic)?
            .parse()
            .map_err(|_| ParserError::Generic)?;
        let second = std::str::from_utf8(&s[4..6])
            .map_err(|_| ParserError::Generic)?
            .parse()
            .map_err(|_| ParserError::Generic)?;
        return Ok(ParserOutput::with_output(
            &s[..6],
            &s[6..],
            Time::new(Some(hour), Some(minute), Some(second)),
        ));
    }
    Err(ParserError::Generic)
}

// "-"  minute second
fn parse_time_minute_second_for_v40(s: &[u8]) -> ParserResult<'_, Time> {
    if s.starts_with(b"-") && s.len() >= 5 && s[1..5].iter().all(|b| b.is_ascii_digit()) {
        let minute = std::str::from_utf8(&s[1..3])
            .map_err(|_| ParserError::Generic)?
            .parse()
            .map_err(|_| ParserError::Generic)?;
        let second = std::str::from_utf8(&s[3..5])
            .map_err(|_| ParserError::Generic)?
            .parse()
            .map_err(|_| ParserError::Generic)?;
        return Ok(ParserOutput::with_output(
            &s[..5],
            &s[5..],
            Time::new(None, Some(minute), Some(second)),
        ));
    }
    Err(ParserError::Generic)
}

// "-" minute
fn parse_time_minute_for_v40(s: &[u8]) -> ParserResult<'_, Time> {
    if s.starts_with(b"-") && s.len() >= 3 && s[1..3].iter().all(|b| b.is_ascii_digit()) {
        let minute = std::str::from_utf8(&s[1..3])
            .map_err(|_| ParserError::Generic)?
            .parse()
            .map_err(|_| ParserError::Generic)?;
        return Ok(ParserOutput::with_output(
            &s[..3],
            &s[3..],
            Time::new(None, Some(minute), None),
        ));
    }
    Err(ParserError::Generic)
}

// "-"   "-"    second
fn parse_time_second_for_v40(s: &[u8]) -> ParserResult<'_, Time> {
    if s.starts_with(b"--") && s.len() >= 4 && s[2..4].iter().all(|b| b.is_ascii_digit()) {
        let second = std::str::from_utf8(&s[2..4])
            .map_err(|_| ParserError::Generic)?
            .parse()
            .map_err(|_| ParserError::Generic)?;
        return Ok(ParserOutput::with_output(
            &s[..4],
            &s[4..],
            Time::new(None, None, Some(second)),
        ));
    }
    Err(ParserError::Generic)
}

// hour [minute [second]] [zone] / "-" minute [second] [zone] / "-" "-" second [zone]
fn parse_time_for_v40(s: &[u8]) -> ParserResult<'_, TimeWithZone> {
    let time = parse_time_hour_minute_second(s)
        .or_else(|_| parse_time_hour_minute_for_v40(s))
        .or_else(|_| parse_time_hour_for_v40(s))
        .or_else(|_| parse_time_minute_second_for_v40(s))
        .or_else(|_| parse_time_minute_for_v40(s))
        .or_else(|_| parse_time_second_for_v40(s))?;

    let zone_res = parse_zone_iso8601_basic(time.remaining()).unwrap_or(ParserOutput::with_output(
        &[],
        time.remaining(),
        None,
    ));

    Ok(ParserOutput::with_output(
        &s[..time.matched().len() + zone_res.matched().len()],
        zone_res.remaining(),
        TimeWithZone::new(time.into_output(), zone_res.into_output()),
    ))
}

// hour [minute [second]] [zone]
fn parse_time_notrunc_for_v40(s: &[u8]) -> ParserResult<'_, TimeWithZone> {
    let time = parse_time_hour_minute_second(s)
        .or_else(|_| parse_time_hour_minute_for_v40(s))
        .or_else(|_| parse_time_hour_for_v40(s))?;

    let zone_res = parse_zone_iso8601_basic(time.remaining()).unwrap_or(ParserOutput::with_output(
        &[],
        time.remaining(),
        None,
    ));

    Ok(ParserOutput::with_output(
        &s[..time.matched().len() + zone_res.matched().len()],
        zone_res.remaining(),
        TimeWithZone::new(time.into_output(), zone_res.into_output()),
    ))
}

// utc-designator = %x5A  ; uppercase "Z"
fn parse_zone_utc(s: &[u8]) -> ParserResult<'_, i32> {
    if s.starts_with(b"Z") {
        return Ok(ParserOutput::with_output(&s[..1], &s[1..], 0));
    }
    Err(ParserError::Generic)
}

// sign digit digit ":" digit digit
fn parse_zone_sign_hh_mm(s: &[u8]) -> ParserResult<'_, i32> {
    if s.len() >= 6
        && (s.starts_with(b"+") || s.starts_with(b"-"))
        && &s[3..4] == b":"
        && s[1..3].iter().all(|b| b.is_ascii_digit())
        && s[4..6].iter().all(|b| b.is_ascii_digit())
    {
        let sign = if s.starts_with(b"+") { 1 } else { -1 };
        let hour: i32 = std::str::from_utf8(&s[1..3])
            .map_err(|_| ParserError::Generic)?
            .parse()
            .map_err(|_| ParserError::Generic)?;
        let minute: i32 = std::str::from_utf8(&s[4..6])
            .map_err(|_| ParserError::Generic)?
            .parse()
            .map_err(|_| ParserError::Generic)?;
        return Ok(ParserOutput::with_output(
            &s[..6],
            &s[6..],
            sign * (hour * 60 + minute),
        ));
    }
    Err(ParserError::Generic)
}

// Z / sign digit digit ":" digit digit
fn parse_zone_iso8601_extended(s: &[u8]) -> ParserResult<'_, Option<i32>> {
    parse_zone_utc(s)
        .map(|res| {
            ParserOutput::with_output(res.matched(), res.remaining(), Some(res.into_output()))
        })
        .or_else(|_| {
            parse_zone_sign_hh_mm(s).map(|res| {
                ParserOutput::with_output(res.matched(), res.remaining(), Some(res.into_output()))
            })
        })
        .or_else(|_| Ok(ParserOutput::with_output(&[], s, None)))
}

// zone = utc-designator / utc-offset
fn parse_zone_iso8601_basic(s: &[u8]) -> ParserResult<'_, Option<i32>> {
    parse_zone_utc(s)
        .map(|res| {
            ParserOutput::with_output(res.matched(), res.remaining(), Some(res.into_output()))
        })
        .or_else(|_| {
            parse_zone_utc_offset_iso8601_basic(s).map(|res| {
                ParserOutput::with_output(res.matched(), res.remaining(), Some(res.into_output()))
            })
        })
}

// ( "+" / "-" ) hour [minute]
fn parse_zone_utc_offset_iso8601_basic(s: &[u8]) -> ParserResult<'_, i32> {
    if s.len() < 3 {
        return Err(ParserError::Generic);
    }
    let sign = if s.starts_with(b"+") { 1 } else { -1 };
    let hour = parse_zone_hour_or_minute(&s[1..])?;
    let minute_result = parse_zone_hour_or_minute(hour.remaining());

    match minute_result {
        Ok(minute) => {
            let matched_len = 1 + hour.matched().len() + minute.matched().len();
            Ok(ParserOutput::with_output(
                &s[..matched_len],
                &s[matched_len..],
                sign * (hour.output() * 60 + minute.output()),
            ))
        }
        Err(_) => {
            let matched_len = 1 + hour.matched().len();
            Ok(ParserOutput::with_output(
                &s[..matched_len],
                &s[matched_len..],
                sign * hour.output() * 60,
            ))
        }
    }
}

// 2DIGIT
fn parse_zone_hour_or_minute(s: &[u8]) -> ParserResult<'_, i32> {
    if s.len() < 2 || !s[..2].iter().all(|b| b.is_ascii_digit()) {
        return Err(ParserError::Generic);
    }

    std::str::from_utf8(&s[..2])
        .map_err(|_| ParserError::Generic)?
        .parse::<i32>()
        .map(|v| ParserOutput::with_output(&s[..2], &s[2..], v))
        .map_err(|_| ParserError::Generic)
}
