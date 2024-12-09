use time::format_description::FormatItem;
use time::macros::format_description;

/// The format used by the `Date` header.
pub const ISO8601: &[FormatItem<'static>] =
    format_description!("[year][month][day]T[hour][minute][second]Z");

/// The format used by the `x-amz-date` header.
#[cfg(feature = "full")]
pub const ISO8601_EXT: &[FormatItem<'static>] =
    format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]Z");

/// The format used by the `x-amz-date` header.
pub const YYYYMMDD: &[FormatItem<'static>] = format_description!("[year][month][day]");
