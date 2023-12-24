use chrono::{Datelike, NaiveDate};

mod bytes;
pub use bytes::*;

pub fn get_erf_date() -> (u32, u32) {
    let now = chrono::Utc::now().date_naive();

    let past = NaiveDate::from_ymd_opt(1900, 1, 1).unwrap();
    let build_year = now.years_since(past).unwrap();

    let past = NaiveDate::from_ymd_opt(now.year_ce().1 as i32, 1, 1).unwrap();
    let build_day = (now - past).num_days() as u32;

    (build_year, build_day - 1)
}

pub fn seconds_to_time(secs: u32) -> String {
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
