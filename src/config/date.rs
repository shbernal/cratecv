//! When the document says it was made.
//!
//! The default is the resume file's modification time, not the clock: `now`
//! would make two builds of the same resume differ, and the guarantee that they
//! do not is worth more than a fresh timestamp.

use std::path::Path;
use std::time::UNIX_EPOCH;

use typst_library::foundations::Datetime;

use crate::diagnostic::Diagnostic;

/// What to stamp the document with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Stamp {
    /// The resume file's modification time.
    Mtime,
    /// No date at all. Conspicuous on a document claiming an origin that always
    /// stamps one, which is the caller's decision to make.
    None,
    /// A timestamp stated in the settings, normalized to UTC.
    Fixed(Datetime),
}

/// Read a `pdf.date` value.
pub fn parse_date(value: &str) -> Result<Stamp, Diagnostic> {
    match value {
        "mtime" => Ok(Stamp::Mtime),
        "none" => Ok(Stamp::None),
        _ => timestamp(value)
            .map(Stamp::Fixed)
            .ok_or_else(|| Diagnostic {
                path: "pdf.date".into(),
                line: 1,
                column: 1,
                message: format!(
                    "`{value}` is not a date. Use `mtime`, `none`, or a quoted timestamp \
                 such as \"2024-03-11T09:12:00Z\""
                ),
            }),
    }
}

impl Stamp {
    /// The datetime to write, given the resume this run is building.
    pub fn resolve(self, input: &Path) -> Option<Datetime> {
        match self {
            Stamp::None => None,
            Stamp::Fixed(when) => Some(when),
            Stamp::Mtime => {
                let seconds = std::fs::metadata(input)
                    .and_then(|meta| meta.modified())
                    .ok()?
                    .duration_since(UNIX_EPOCH)
                    .ok()?
                    .as_secs() as i64;
                from_unix(seconds)
            }
        }
    }
}

/// `YYYY-MM-DD`, or a full timestamp with `Z` or a `±HH:MM` offset, which is
/// applied so the stored value is UTC.
fn timestamp(value: &str) -> Option<Datetime> {
    let (date, rest) = value.split_at_checked(10)?;
    let mut parts = date.split('-');
    let year: i32 = parts.next()?.parse().ok()?;
    let month: u8 = parts.next()?.parse().ok()?;
    let day: u8 = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    if rest.is_empty() {
        return Datetime::from_ymd(year, month, day);
    }

    let rest = rest.strip_prefix('T').or_else(|| rest.strip_prefix(' '))?;
    let (clock, zone) = match rest.find(['Z', '+']) {
        Some(at) => rest.split_at(at),
        None => match rest[1..].find('-') {
            Some(at) => rest.split_at(at + 1),
            None => (rest, ""),
        },
    };
    let mut parts = clock.split(':');
    let hour: u8 = parts.next()?.parse().ok()?;
    let minute: u8 = parts.next()?.parse().ok()?;
    let second: u8 = match parts.next() {
        Some(text) => text.parse().ok()?,
        None => 0,
    };

    let offset = zone_minutes(zone)?;
    let unix = to_unix(year, month, day, hour, minute, second) - offset * 60;
    from_unix(unix)
}

fn zone_minutes(zone: &str) -> Option<i64> {
    if zone.is_empty() || zone == "Z" {
        return Some(0);
    }
    let sign = match zone.as_bytes().first()? {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    let mut parts = zone[1..].split(':');
    let hours: i64 = parts.next()?.parse().ok()?;
    let minutes: i64 = match parts.next() {
        Some(text) => text.parse().ok()?,
        None => 0,
    };
    Some(sign * (hours * 60 + minutes))
}

/// Days since the epoch for a civil date, and back. Hinnant's algorithm, which
/// is short enough to own and saves a calendar dependency in a static binary.
fn days_from_civil(year: i32, month: u8, day: u8) -> i64 {
    let year = year as i64 - i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month = month as i64;
    let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day as i64 - 1;
    let doe = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(days: i64) -> (i32, u8, u8) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let doe = days - era * 146_097;
    let year_of_era = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let year = year_of_era + era * 400;
    let doy = doe - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u8;
    let month = (mp + if mp < 10 { 3 } else { -9 }) as u8;
    ((year + i64::from(month <= 2)) as i32, month, day)
}

fn to_unix(year: i32, month: u8, day: u8, hour: u8, minute: u8, second: u8) -> i64 {
    days_from_civil(year, month, day) * 86_400
        + hour as i64 * 3_600
        + minute as i64 * 60
        + second as i64
}

fn from_unix(seconds: i64) -> Option<Datetime> {
    let days = seconds.div_euclid(86_400);
    let rest = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    Datetime::from_ymd_hms(
        year,
        month,
        day,
        (rest / 3_600) as u8,
        ((rest % 3_600) / 60) as u8,
        (rest % 60) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bare_date_is_a_date() {
        let Stamp::Fixed(when) = parse_date("2024-03-11").unwrap() else {
            panic!("a date is a fixed stamp");
        };
        assert_eq!(
            (when.year(), when.month(), when.day()),
            (Some(2024), Some(3), Some(11))
        );
    }

    #[test]
    fn an_offset_is_applied_rather_than_dropped() {
        let Stamp::Fixed(when) = parse_date("2024-03-11T09:12:00+01:00").unwrap() else {
            panic!("a timestamp is a fixed stamp");
        };
        assert_eq!(
            when.hour(),
            Some(8),
            "01:00 east of UTC is an hour earlier in UTC"
        );
        assert_eq!(when.minute(), Some(12));
    }

    #[test]
    fn a_negative_offset_is_applied_too() {
        let Stamp::Fixed(when) = parse_date("2024-03-11T09:12:00-05:00").unwrap() else {
            panic!("a timestamp is a fixed stamp");
        };
        assert_eq!(when.hour(), Some(14));
    }

    #[test]
    fn the_two_words_are_not_dates() {
        assert_eq!(parse_date("mtime").unwrap(), Stamp::Mtime);
        assert_eq!(parse_date("none").unwrap(), Stamp::None);
    }

    #[test]
    fn nonsense_says_what_it_wanted() {
        let why = parse_date("last tuesday").unwrap_err();
        assert!(why.message.contains("mtime"), "{}", why.message);
    }

    #[test]
    fn the_calendar_round_trips() {
        for unix in [0_i64, 1_000_000_000, 1_732_060_800, -86_400] {
            let when = from_unix(unix).expect("a representable datetime");
            assert_eq!(
                to_unix(
                    when.year().unwrap(),
                    when.month().unwrap(),
                    when.day().unwrap(),
                    when.hour().unwrap(),
                    when.minute().unwrap(),
                    when.second().unwrap(),
                ),
                unix
            );
        }
    }
}
