//! Calendar calculation logic using Zeller's algorithm and custom reform handling.

use chrono::{Datelike, NaiveDate, Weekday};

use crate::types::{
    CELLS_PER_MONTH, CalContext, ColumnsMode, MonthData, REFORM_FIRST_DAY, REFORM_LAST_DAY,
    REFORM_MONTH, REFORM_YEAR_GB, WeekType,
};

impl CalContext {
    /// Check if a year is a leap year according to the calendar rules.
    pub fn is_leap_year(&self, year: i32) -> bool {
        if year < self.reform_year {
            // Julian: every 4 years
            year % 4 == 0
        } else {
            // Gregorian: divisible by 4, except centuries unless divisible by 400
            (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
        }
    }

    pub fn days_in_month(&self, year: i32, month: u32) -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if self.is_leap_year(year) => 29,
            2 => 28,
            _ => 30,
        }
    }

    /// Check if a date falls within the reform gap (September 3-13, 1752).
    pub fn is_reform_gap(&self, year: i32, month: u32, day: u32) -> bool {
        if self.reform_year != REFORM_YEAR_GB {
            return false;
        }
        year == REFORM_YEAR_GB
            && month == REFORM_MONTH
            && (REFORM_FIRST_DAY..=REFORM_LAST_DAY).contains(&day)
    }

    /// Calculate weekday using Zeller's congruence algorithm.
    pub fn first_day_of_month(&self, year: i32, month: u32) -> Weekday {
        let m = if month < 3 { month + 12 } else { month };
        let q: i32 = 1;
        let year_i = if month < 3 { year - 1 } else { year };
        let k: i32 = year_i % 100;
        let j: i32 = year_i / 100;

        let h = if year < self.reform_year {
            // Julian calendar: no century correction
            (q + (13 * (m as i32 + 1)) / 5 + k + k / 4 + 5).rem_euclid(7)
        } else {
            // Gregorian calendar
            (q + (13 * (m as i32 + 1)) / 5 + k + k / 4 + j / 4 - 2 * j).rem_euclid(7)
        };
        // h: 0=Sat, 1=Sun, 2=Mon, 3=Tue, 4=Wed, 5=Thu, 6=Fri
        match h {
            0 => Weekday::Sat,
            1 => Weekday::Sun,
            2 => Weekday::Mon,
            3 => Weekday::Tue,
            4 => Weekday::Wed,
            5 => Weekday::Thu,
            6 => Weekday::Fri,
            _ => unreachable!(),
        }
    }

    /// Calculate day of year (Julian day number within the year).
    pub fn day_of_year(&self, year: i32, month: u32, day: u32) -> u32 {
        const DAYS_BEFORE_MONTH: [u32; 12] =
            [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
        let mut doy = DAYS_BEFORE_MONTH[(month - 1) as usize] + day;

        if month > 2 && self.is_leap_year(year) {
            doy += 1;
        }

        // Adjust for reform gap (11 days removed in September 1752)
        if year == REFORM_YEAR_GB && month >= REFORM_MONTH {
            doy = doy.saturating_sub(REFORM_LAST_DAY - REFORM_FIRST_DAY + 1);
        }
        doy
    }

    pub fn week_number(&self, year: i32, month: u32, day: u32) -> u32 {
        match self.week_type {
            WeekType::Iso => {
                // ISO 8601: week starts Monday, week 1 contains first Thursday
                let date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
                date.iso_week().week()
            }
            WeekType::Us => {
                // US: week starts Sunday, week 1 contains January 1
                let date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
                let jan1 = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
                let days_since_jan1 = date.signed_duration_since(jan1).num_days() as u32;
                let jan1_weekday = jan1.weekday().num_days_from_sunday();
                ((days_since_jan1 + jan1_weekday) / 7) + 1
            }
        }
    }

    pub fn is_weekend(&self, weekday: Weekday) -> bool {
        matches!(weekday, Weekday::Sat | Weekday::Sun)
    }

    pub fn months_per_row(&self) -> u32 {
        match self.columns {
            ColumnsMode::Fixed(n) => n,
            ColumnsMode::Auto => {
                // ~20 chars per month + gutter, clamp to 1-3 for readability
                let month_width = 20 + self.gutter_width;
                if let Some(term_width) = get_terminal_width() {
                    (term_width / month_width as u32).clamp(1, 3)
                } else {
                    3
                }
            }
        }
    }
}

impl MonthData {
    /// Build calendar data for a specific month.
    pub fn new(ctx: &CalContext, year: i32, month: u32) -> Self {
        let days_in_month = ctx.days_in_month(year, month);
        let first_day = ctx.first_day_of_month(year, month);

        // Calculate offset based on week start day
        let offset = match ctx.week_start {
            Weekday::Mon if first_day == Weekday::Sun => 6,
            Weekday::Mon => first_day.num_days_from_monday() as usize,
            Weekday::Sun => first_day.num_days_from_sunday() as usize,
            _ => unreachable!(),
        };

        let mut days: Vec<Option<u32>> = Vec::with_capacity(CELLS_PER_MONTH);
        let mut week_numbers: Vec<Option<u32>> = Vec::with_capacity(CELLS_PER_MONTH);
        let mut weekdays: Vec<Option<Weekday>> = Vec::with_capacity(CELLS_PER_MONTH);

        // Empty cells before first day
        for _ in 0..offset {
            days.push(None);
            week_numbers.push(None);
            weekdays.push(None);
        }

        // Fill days, skipping reform gap
        let mut current_weekday = first_day;
        let mut day = 1;
        while day <= days_in_month {
            if ctx.is_reform_gap(year, month, day) {
                // Skip reform gap (3-13 September 1752)
                for _ in REFORM_FIRST_DAY..=REFORM_LAST_DAY {
                    days.push(None);
                    week_numbers.push(None);
                    weekdays.push(None);
                    current_weekday = current_weekday.succ();
                }
                day = REFORM_LAST_DAY + 1;
            } else {
                days.push(Some(day));
                week_numbers.push(ctx.week_numbers.then(|| ctx.week_number(year, month, day)));
                weekdays.push(Some(current_weekday));
                current_weekday = current_weekday.succ();
                day += 1;
            }
        }

        // Pad to 42 cells (6 weeks)
        while days.len() < CELLS_PER_MONTH {
            days.push(None);
            week_numbers.push(None);
            weekdays.push(None);
        }

        MonthData {
            year,
            month,
            days,
            week_numbers,
            weekdays,
        }
    }
}

/// Get terminal width using terminal_size crate.
fn get_terminal_width() -> Option<u32> {
    terminal_size::terminal_size().map(|(w, _)| w.0 as u32)
}
