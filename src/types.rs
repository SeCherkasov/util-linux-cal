//! Type definitions and constants for calendar formatting.

use chrono::Weekday;
use clap::ValueEnum;

/// Calendar reform type determining which calendar system to use.
#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum ReformType {
    /// Gregorian calendar (always).
    Gregorian,
    /// ISO 8601 calendar (same as Gregorian).
    Iso,
    /// Julian calendar (always).
    Julian,
    /// Great Britain reform year 1752 (September 3-13 were skipped).
    #[value(name = "1752")]
    Year1752,
}

impl ReformType {
    /// Return the reform year for this calendar type.
    pub fn reform_year(self) -> i32 {
        match self {
            ReformType::Gregorian | ReformType::Iso => i32::MIN,
            ReformType::Julian => i32::MAX,
            ReformType::Year1752 => REFORM_YEAR_GB,
        }
    }
}

/// Week numbering system for calendar display.
#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
pub enum WeekType {
    /// ISO 8601: week starts on Monday, week 1 contains the first Thursday.
    Iso,
    /// US style: week starts on Sunday, week 1 contains January 1.
    Us,
}

/// Column display mode for multi-month layouts.
#[derive(Debug, Clone, Copy)]
pub enum ColumnsMode {
    /// Fixed number of columns.
    Fixed(u32),
    /// Auto-detect from terminal width.
    Auto,
}

/// Calendar formatting context containing all display options.
#[derive(Clone, Debug)]
pub struct CalContext {
    /// Year when calendar reform occurred (i32::MIN = always Gregorian, i32::MAX = always Julian).
    pub reform_year: i32,
    /// First day of the week (Monday or Sunday).
    pub week_start: Weekday,
    /// Whether to display Julian day numbers (day of year).
    pub julian: bool,
    /// Whether to display ISO week numbers.
    pub week_numbers: bool,
    /// Week numbering system (ISO or US).
    pub week_type: WeekType,
    /// Whether to use ANSI color codes in output.
    pub color: bool,
    /// Whether to display days vertically (days in columns instead of rows).
    pub vertical: bool,
    /// Today's date for highlighting.
    pub today: chrono::NaiveDate,
    /// Whether to show year in month headers.
    pub show_year_in_header: bool,
    /// Width of gutter between months in multi-month display.
    pub gutter_width: usize,
    /// Column display mode.
    pub columns: ColumnsMode,
    /// Whether to center the date range when displaying multiple months.
    pub span: bool,
    /// Whether to highlight holidays using isdayoff.ru API.
    #[cfg(feature = "plugins")]
    pub holidays: bool,
}

/// Calendar data for a single month.
pub struct MonthData {
    pub year: i32,
    pub month: u32,
    pub days: Vec<Option<u32>>,
    pub week_numbers: Vec<Option<u32>>,
    pub weekdays: Vec<Option<Weekday>>,
}

// Constants for calendar formatting
pub const CELLS_PER_MONTH: usize = 42; // 6 weeks × 7 days
pub const GUTTER_WIDTH_REGULAR: usize = 2;
pub const GUTTER_WIDTH_YEAR: usize = 3;

// Color is enabled by default for better user experience
pub const COLOR_ENABLED_BY_DEFAULT: bool = true;

// Reform year for September 1752 (missing days 3-13 in Great Britain)
pub const REFORM_YEAR_GB: i32 = 1752;
pub const REFORM_MONTH: u32 = 9;
pub const REFORM_FIRST_DAY: u32 = 3;
pub const REFORM_LAST_DAY: u32 = 13;

// ANSI color codes
pub const COLOR_RESET: &str = "\x1b[0m";
pub const COLOR_REVERSE: &str = "\x1b[7m";
pub const COLOR_RED: &str = "\x1b[91m";
pub const COLOR_TEAL: &str = "\x1b[96m";
pub const COLOR_SAND_YELLOW: &str = "\x1b[93m";
