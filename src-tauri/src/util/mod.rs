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
