use time::{macros::datetime, OffsetDateTime};

mod bytes;
mod ui;
pub use bytes::*;
pub use ui::*;

// years since 1900 and days since jan 1
pub fn get_erf_date() -> (u32, u32) {
    let now = OffsetDateTime::now_utc();
    let past = datetime!(1900 - 01 - 01 0:00 UTC);
    let build_year = now.year() - past.year();

    past.replace_year(now.year()).unwrap();
    let build_day = (now - past).whole_days();

    (build_year as u32, build_day as u32)
}

pub fn format_seconds(secs: u32) -> String {
    let seconds = secs % 60;
    let minutes = secs / 60 % 60;
    let hours = secs / 60 / 60 % 24;
    let days = secs / 60 / 60 / 24;

    if days > 0 {
        format!("{days}d {hours}h {minutes}m {seconds}s")
    } else if hours > 0 {
        format!("{hours}h {minutes}m {seconds}s")
    } else {
        format!("{minutes}m {seconds}s")
    }
}
