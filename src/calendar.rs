// SPDX-License-Identifier: MPL-2.0

use chrono::{Datelike, Local, NaiveDate};

pub fn today_string() -> String {
    Local::now().date_naive().format("%Y-%m-%d").to_string()
}

/// Returns the first day of the month as a NaiveDate, or None if invalid.
pub fn first_of_month(year: i32, month: u32) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(year, month, 1)
}

/// Returns all days in the given month.
pub fn days_in_month(year: i32, month: u32) -> Vec<NaiveDate> {
    let mut days = Vec::new();
    let mut day = 1u32;
    while let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
        days.push(date);
        day += 1;
    }
    days
}

/// Returns (year, month) for the previous month.
pub fn prev_month(year: i32, month: u32) -> (i32, u32) {
    if month == 1 {
        (year - 1, 12)
    } else {
        (year, month - 1)
    }
}

/// Returns (year, month) for the next month.
pub fn next_month(year: i32, month: u32) -> (i32, u32) {
    if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    }
}

pub fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "???",
    }
}

/// Returns the weekday index (0 = Monday … 6 = Sunday) of the first day of the month.
pub fn month_start_weekday(year: i32, month: u32) -> usize {
    first_of_month(year, month)
        .map(|d| d.weekday().num_days_from_monday() as usize)
        .unwrap_or(0)
}

pub fn date_string(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}
