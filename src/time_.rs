use time::format_description::FormatItem;
use time::macros::format_description;

pub const ISO8601: &[FormatItem<'static>] =
    format_description!("[year][month][day]T[hour][minute][second]Z");
pub const ISO8601_EXT: &[FormatItem<'static>] =
    format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]Z");
pub const YYYYMMDD: &[FormatItem<'static>] = format_description!("[year][month][day]");
