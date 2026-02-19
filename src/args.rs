//! Command-line argument parsing using clap.
//!
//! Arguments follow util-linux cal convention: `[[day] month] year`

use chrono::Datelike;
use clap::{Parser, ValueHint};
use std::io::IsTerminal;

use crate::types::{
    COLOR_ENABLED_BY_DEFAULT, CalContext, ColumnsMode, GUTTER_WIDTH_REGULAR, ReformType, WeekType,
};

#[derive(Parser, Debug)]
#[command(name = "cal")]
#[command(about = "Displays calendar for specified month or year", long_about = None)]
#[command(version)]
#[command(after_help = HELP_MESSAGE)]
pub struct Args {
    /// Week starts on Sunday (default is Monday).
    #[arg(short = 's', long, help_heading = "Calendar options")]
    pub sunday: bool,

    /// Week starts on Monday (default).
    #[arg(short = 'm', long, help_heading = "Calendar options")]
    pub monday: bool,

    /// Display Julian days (day number in year).
    #[arg(short = 'j', long, help_heading = "Calendar options")]
    pub julian: bool,

    /// Display week numbers.
    #[arg(short = 'w', long, help_heading = "Calendar options")]
    pub week_numbers: bool,

    /// Week numbering system (iso or us).
    #[arg(
        long,
        default_value = "iso",
        help_heading = "Calendar options",
        value_name = "system"
    )]
    pub week_type: WeekType,

    /// Display whole year.
    #[arg(short = 'y', long, help_heading = "Display options")]
    pub year: bool,

    /// Display the next twelve months.
    #[arg(short = 'Y', long = "twelve", help_heading = "Display options")]
    pub twelve_months: bool,

    /// Display three months (previous, current, next).
    #[arg(short = '3', long = "three", help_heading = "Display options")]
    pub three_months: bool,

    /// Number of months to display.
    #[arg(
        short = 'n',
        long = "months",
        help_heading = "Display options",
        value_name = "num"
    )]
    pub months_count: Option<u32>,

    /// Show only a single month (default).
    #[arg(short = '1', long = "one", help_heading = "Display options")]
    pub one_month: bool,

    /// Span the date when displaying multiple months (center around current month).
    #[arg(short = 'S', long = "span", help_heading = "Display options")]
    pub span: bool,

    /// Gregorian reform date (1752|gregorian|iso|julian).
    #[arg(
        long,
        default_value = "1752",
        help_heading = "Calendar options",
        value_name = "val"
    )]
    pub reform: ReformType,

    /// Use ISO 8601 reform (same as --reform iso).
    #[arg(long, help_heading = "Calendar options")]
    pub iso: bool,

    /// Day (1-31) - optional, used with month and year.
    #[arg(index = 1, default_value = None, value_name = "day", value_hint = ValueHint::Other)]
    pub day_arg: Option<String>,

    /// Month (1-12 or name) - optional, used with year.
    #[arg(index = 2, default_value = None, value_name = "month", value_hint = ValueHint::Other)]
    pub month_arg: Option<String>,

    /// Year (1-9999).
    #[arg(index = 3, default_value = None, value_name = "year", value_hint = ValueHint::Other)]
    pub year_arg: Option<String>,

    /// Disable colorized output.
    #[arg(long, help_heading = "Output options")]
    pub color: bool,

    /// Number of columns for multiple months (or "auto" for terminal width).
    #[arg(
        short = 'c',
        long = "columns",
        help_heading = "Output options",
        value_name = "width"
    )]
    pub columns: Option<String>,

    /// Show days vertically (days in columns instead of rows).
    #[arg(short = 'v', long, help_heading = "Output options")]
    pub vertical: bool,

    /// Highlight holidays using isdayoff.ru API (requires plugin).
    ///
    /// **Note:** Build the workspace to include the plugin:
    /// ```bash
    /// cargo build --release --workspace
    /// ```
    /// The plugin file (`libholiday_highlighter.so`) must be in one of:
    /// - `./target/release/` (after building)
    /// - `~/.local/lib/cal/plugins/`
    /// - `/usr/lib/cal/plugins/`
    #[arg(short = 'H', long = "holidays", help_heading = "Output options")]
    pub holidays: bool,
}

/// Help message displayed with --help.
const HELP_MESSAGE: &str = "Display a calendar, or some part of it.

Without any arguments, display the current month.

Examples:
  cal                Display current month
  cal -3             Display three months (prev, current, next)
  cal -y             Display the whole year
  cal -Y             Display next twelve months
  cal 2 2026         Display February 2026
  cal 2026           Display year 2026
  cal --span -n 12   Display 12 months centered on current month
  cal --color        Disable colorized output
  cal -H             Highlight holidays (requires plugin, see --help)";

impl Args {
    pub fn parse() -> Self {
        Parser::parse()
    }
}

impl CalContext {
    pub fn new(args: &Args) -> Result<Self, String> {
        let today = get_today_date();

        let color = !args.color && COLOR_ENABLED_BY_DEFAULT && std::io::stdout().is_terminal();

        let columns = match args.columns.as_deref() {
            Some("auto") | None => ColumnsMode::Auto,
            Some(s) => {
                let n = s
                    .parse::<u32>()
                    .map_err(|_| format!("Invalid columns value: {}", s))?;
                if n == 0 {
                    return Err("Columns must be positive".to_string());
                }
                ColumnsMode::Fixed(n)
            }
        };

        // Prevent conflicting display modes
        let mode_count = [args.year, args.twelve_months, args.months_count.is_some()]
            .iter()
            .filter(|&&x| x)
            .count();

        if mode_count > 1 {
            return Err("Options -y, -Y, and -n are mutually exclusive".to_string());
        }

        if let Some(year_str) = &args.year_arg {
            let year: i32 = year_str
                .parse()
                .map_err(|_| format!("Invalid year value: {}", year_str))?;
            if !(1..=9999).contains(&year) {
                return Err(format!("Invalid year value: {} (must be 1-9999)", year));
            }
        }

        // Vertical mode uses narrower gutter for compact layout
        let gutter_width = if args.vertical {
            1
        } else {
            GUTTER_WIDTH_REGULAR
        };

        // --iso overrides --reform
        let reform_year = if args.iso {
            ReformType::Iso.reform_year()
        } else {
            args.reform.reform_year()
        };

        Ok(CalContext {
            reform_year,
            week_start: if args.sunday {
                chrono::Weekday::Sun
            } else {
                chrono::Weekday::Mon
            },
            julian: args.julian,
            week_numbers: args.week_numbers,
            week_type: args.week_type,
            color,
            vertical: args.vertical,
            today,
            show_year_in_header: true,
            gutter_width,
            columns,
            span: args.span,
            #[cfg(feature = "plugins")]
            holidays: args.holidays,
        })
    }
}

/// Get today's date, respecting CAL_TEST_TIME environment variable for testing.
pub fn get_today_date() -> chrono::NaiveDate {
    if let Ok(test_time) = std::env::var("CAL_TEST_TIME")
        && let Ok(date) = chrono::NaiveDate::parse_from_str(&test_time, "%Y-%m-%d")
    {
        return date;
    }
    chrono::Local::now().date_naive()
}

/// Calculate display date from positional arguments.
///
/// Argument patterns:
/// - 1 arg: year (4 digits) or month (1-2 digits)
/// - 2 args: month year
/// - 3 args: day month year
pub fn get_display_date(args: &Args) -> Result<(i32, u32, Option<u32>), String> {
    let today = get_today_date();

    let day_provided = args.day_arg.is_some();
    let month_provided = args.month_arg.is_some();
    let year_provided = args.year_arg.is_some();

    match (day_provided, month_provided, year_provided) {
        // One argument: could be year (4 digits) or month (1-2 digits)
        (true, false, false) => {
            let val = args.day_arg.as_ref().unwrap();
            if let Ok(num) = val.parse::<i32>() {
                // 4 digits = year
                if (1000..=9999).contains(&num) {
                    return Ok((num, today.month(), None));
                }
                // 1-2 digits = month
                if (1..=12).contains(&num) {
                    return Ok((today.year(), num as u32, None));
                }
            }
            // Try parsing as month name
            if let Some(month) = crate::formatter::parse_month(val) {
                return Ok((today.year(), month, None));
            }
            Err(format!("Invalid argument: {}", val))
        }
        // Two arguments: month year (e.g., cal 2 2026)
        (true, true, false) => {
            let month = crate::formatter::parse_month(args.day_arg.as_ref().unwrap())
                .ok_or_else(|| format!("Invalid month: {}", args.day_arg.as_ref().unwrap()))?;
            let year = args
                .month_arg
                .as_ref()
                .unwrap()
                .parse::<i32>()
                .map_err(|_| format!("Invalid year: {}", args.month_arg.as_ref().unwrap()))?;
            if !(1..=9999).contains(&year) {
                return Err(format!("Invalid year: {} (must be 1-9999)", year));
            }
            Ok((year, month, None))
        }
        // Three arguments: day month year
        (true, true, true) => {
            let day = args
                .day_arg
                .as_ref()
                .unwrap()
                .parse::<u32>()
                .map_err(|_| format!("Invalid day: {}", args.day_arg.as_ref().unwrap()))?;
            if !(1..=31).contains(&day) {
                return Err(format!("Invalid day: {} (must be 1-31)", day));
            }
            let month = crate::formatter::parse_month(args.month_arg.as_ref().unwrap())
                .ok_or_else(|| format!("Invalid month: {}", args.month_arg.as_ref().unwrap()))?;
            let year = args
                .year_arg
                .as_ref()
                .unwrap()
                .parse::<i32>()
                .map_err(|_| format!("Invalid year: {}", args.year_arg.as_ref().unwrap()))?;
            if !(1..=9999).contains(&year) {
                return Err(format!("Invalid year: {} (must be 1-9999)", year));
            }
            Ok((year, month, Some(day)))
        }
        // No arguments: current month
        (false, false, false) => Ok((today.year(), today.month(), None)),
        // Invalid combinations
        _ => Err("Invalid argument combination".to_string()),
    }
}
