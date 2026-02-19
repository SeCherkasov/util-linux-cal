//! Unit tests for calendar calculation logic, formatting, and argument parsing.

use std::io::IsTerminal;

use chrono::{Datelike, Weekday};
use unicode_width::UnicodeWidthStr;

use cal::args::{Args, get_display_date};
use cal::formatter::{
    format_month_grid, format_month_header, format_weekday_headers, get_weekday_order, parse_month,
};
use cal::types::{CalContext, ColumnsMode, MonthData, ReformType, WeekType};

use clap::Parser;

// ---------------------------------------------------------------------------
// Test context helpers
// ---------------------------------------------------------------------------

fn base_context() -> CalContext {
    CalContext {
        reform_year: ReformType::Year1752.reform_year(),
        week_start: Weekday::Mon,
        julian: false,
        week_numbers: false,
        week_type: WeekType::Iso,
        color: false,
        vertical: false,
        today: chrono::NaiveDate::from_ymd_opt(2026, 2, 18).unwrap(),
        show_year_in_header: true,
        gutter_width: 2,
        columns: ColumnsMode::Auto,
        span: false,
        #[cfg(feature = "plugins")]
        holidays: false,
    }
}

fn julian_context() -> CalContext {
    CalContext {
        reform_year: ReformType::Julian.reform_year(),
        ..base_context()
    }
}

fn gregorian_context() -> CalContext {
    CalContext {
        reform_year: ReformType::Gregorian.reform_year(),
        ..base_context()
    }
}

// ===========================================================================
// Leap year
// ===========================================================================

mod leap_year {
    use super::*;

    #[test]
    fn gregorian_divisible_by_400() {
        let ctx = gregorian_context();
        assert!(ctx.is_leap_year(2000));
        assert!(ctx.is_leap_year(2400));
    }

    #[test]
    fn gregorian_divisible_by_4_not_100() {
        let ctx = gregorian_context();
        assert!(ctx.is_leap_year(2024));
        assert!(ctx.is_leap_year(2028));
        assert!(!ctx.is_leap_year(2023));
        assert!(!ctx.is_leap_year(2025));
    }

    #[test]
    fn gregorian_century_not_leap() {
        let ctx = gregorian_context();
        assert!(!ctx.is_leap_year(1900));
        assert!(!ctx.is_leap_year(2100));
        assert!(!ctx.is_leap_year(2200));
    }

    #[test]
    fn julian_every_4th_year() {
        let ctx = julian_context();
        assert!(ctx.is_leap_year(2024));
        assert!(ctx.is_leap_year(1900)); // Julian: 1900 IS leap
        assert!(ctx.is_leap_year(100));
        assert!(!ctx.is_leap_year(2023));
    }

    #[test]
    fn year_1752_reform_boundary() {
        let ctx = base_context(); // reform_year = 1752
        // 1752 is before reform -> Julian rules -> divisible by 4 -> leap
        assert!(ctx.is_leap_year(1752));
        // 1751 not divisible by 4
        assert!(!ctx.is_leap_year(1751));
    }
}

// ===========================================================================
// Days in month
// ===========================================================================

mod days_in_month {
    use super::*;

    #[test]
    fn months_with_31_days() {
        let ctx = base_context();
        for month in [1, 3, 5, 7, 8, 10, 12] {
            assert_eq!(ctx.days_in_month(2024, month), 31, "month {month}");
        }
    }

    #[test]
    fn months_with_30_days() {
        let ctx = base_context();
        for month in [4, 6, 9, 11] {
            assert_eq!(ctx.days_in_month(2024, month), 30, "month {month}");
        }
    }

    #[test]
    fn february_leap() {
        let ctx = base_context();
        assert_eq!(ctx.days_in_month(2024, 2), 29);
        assert_eq!(ctx.days_in_month(2000, 2), 29);
    }

    #[test]
    fn february_non_leap() {
        let ctx = base_context();
        assert_eq!(ctx.days_in_month(2023, 2), 28);
        assert_eq!(ctx.days_in_month(2025, 2), 28);
    }
}

// ===========================================================================
// First day of month (Zeller's congruence)
// ===========================================================================

mod first_day_of_month {
    use super::*;

    #[test]
    fn known_gregorian_dates() {
        let ctx = base_context();
        assert_eq!(ctx.first_day_of_month(2024, 1), Weekday::Mon);
        assert_eq!(ctx.first_day_of_month(2025, 1), Weekday::Wed);
        assert_eq!(ctx.first_day_of_month(2024, 2), Weekday::Thu);
        assert_eq!(ctx.first_day_of_month(2026, 2), Weekday::Sun);
        assert_eq!(ctx.first_day_of_month(2000, 1), Weekday::Sat);
    }

    #[test]
    fn september_1752_reform() {
        let ctx = base_context();
        assert_eq!(ctx.first_day_of_month(1752, 9), Weekday::Fri);
    }

    #[test]
    fn julian_calendar_dates() {
        let ctx = julian_context();
        // Under pure Julian, 1900 is a leap year (divisible by 4).
        // Julian Zeller for 1 March 1900: Monday
        assert_eq!(ctx.first_day_of_month(1900, 3), Weekday::Mon);
        // Julian and Gregorian agree for dates well after reform.
        // Verify that Julian context still computes early dates without panic.
        let _ = ctx.first_day_of_month(500, 6);
    }

    #[test]
    fn gregorian_calendar_dates() {
        let ctx = gregorian_context();
        // Under pure Gregorian, 1 March 1900 is a Thursday.
        let day = ctx.first_day_of_month(1900, 3);
        assert_eq!(day, Weekday::Thu);

        // 1 Jan 2024
        assert_eq!(ctx.first_day_of_month(2024, 1), Weekday::Mon);
    }

    #[test]
    fn january_and_february_use_previous_year_in_formula() {
        let ctx = gregorian_context();
        // January 2023 starts on Sunday
        assert_eq!(ctx.first_day_of_month(2023, 1), Weekday::Sun);
        // February 2023 starts on Wednesday
        assert_eq!(ctx.first_day_of_month(2023, 2), Weekday::Wed);
    }
}

// ===========================================================================
// Reform gap (September 1752)
// ===========================================================================

mod reform_gap {
    use super::*;

    #[test]
    fn days_inside_gap() {
        let ctx = base_context();
        for day in 3..=13 {
            assert!(
                ctx.is_reform_gap(1752, 9, day),
                "day {day} should be in gap"
            );
        }
    }

    #[test]
    fn days_outside_gap() {
        let ctx = base_context();
        assert!(!ctx.is_reform_gap(1752, 9, 2));
        assert!(!ctx.is_reform_gap(1752, 9, 14));
    }

    #[test]
    fn wrong_month_or_year() {
        let ctx = base_context();
        assert!(!ctx.is_reform_gap(1752, 8, 5));
        assert!(!ctx.is_reform_gap(1752, 10, 5));
        assert!(!ctx.is_reform_gap(1751, 9, 5));
        assert!(!ctx.is_reform_gap(2024, 9, 5));
    }

    #[test]
    fn no_gap_in_pure_gregorian() {
        let ctx = gregorian_context();
        assert!(!ctx.is_reform_gap(1752, 9, 5));
    }

    #[test]
    fn no_gap_in_pure_julian() {
        let ctx = julian_context();
        assert!(!ctx.is_reform_gap(1752, 9, 5));
    }
}

// ===========================================================================
// Day of year
// ===========================================================================

mod day_of_year {
    use super::*;

    #[test]
    fn non_leap_year() {
        let ctx = base_context();
        assert_eq!(ctx.day_of_year(2023, 1, 1), 1);
        assert_eq!(ctx.day_of_year(2023, 1, 31), 31);
        assert_eq!(ctx.day_of_year(2023, 2, 1), 32);
        assert_eq!(ctx.day_of_year(2023, 12, 31), 365);
    }

    #[test]
    fn leap_year() {
        let ctx = base_context();
        assert_eq!(ctx.day_of_year(2024, 1, 1), 1);
        assert_eq!(ctx.day_of_year(2024, 2, 29), 60);
        assert_eq!(ctx.day_of_year(2024, 3, 1), 61);
        assert_eq!(ctx.day_of_year(2024, 12, 31), 366);
    }

    #[test]
    fn reform_gap_adjustment() {
        let ctx = base_context();
        // Before gap
        assert_eq!(ctx.day_of_year(1752, 9, 2), 235);
        // After gap: 11 days removed
        assert_eq!(ctx.day_of_year(1752, 9, 14), 247);
    }
}

// ===========================================================================
// Week numbers
// ===========================================================================

mod week_numbers {
    use super::*;

    #[test]
    fn iso_week_jan_1() {
        let mut ctx = base_context();
        ctx.week_type = WeekType::Iso;
        // 2024-01-01 is Monday, ISO week 1
        assert_eq!(ctx.week_number(2024, 1, 1), 1);
    }

    #[test]
    fn iso_week_year_end() {
        let mut ctx = base_context();
        ctx.week_type = WeekType::Iso;
        // 2024-12-30 is Monday — could be week 1 of 2025 or week 53 of 2024
        let wk = ctx.week_number(2024, 12, 30);
        assert!(wk == 1 || wk == 53);
    }

    #[test]
    fn us_week_jan_1() {
        let mut ctx = base_context();
        ctx.week_type = WeekType::Us;
        assert_eq!(ctx.week_number(2024, 1, 1), 1);
    }

    #[test]
    fn us_week_mid_year() {
        let mut ctx = base_context();
        ctx.week_type = WeekType::Us;
        // Sanity: week number grows through the year
        let wk = ctx.week_number(2024, 7, 1);
        assert!(wk > 25);
    }
}

// ===========================================================================
// Weekend detection
// ===========================================================================

mod weekend {
    use super::*;

    #[test]
    fn saturday_and_sunday_are_weekends() {
        let ctx = base_context();
        assert!(ctx.is_weekend(Weekday::Sat));
        assert!(ctx.is_weekend(Weekday::Sun));
    }

    #[test]
    fn weekdays_are_not_weekends() {
        let ctx = base_context();
        for day in [
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
        ] {
            assert!(!ctx.is_weekend(day), "{day:?}");
        }
    }
}

// ===========================================================================
// MonthData construction
// ===========================================================================

mod month_data {
    use super::*;

    #[test]
    fn january_2024_starts_monday() {
        let ctx = base_context();
        let m = MonthData::new(&ctx, 2024, 1);

        assert_eq!(m.year, 2024);
        assert_eq!(m.month, 1);
        assert_eq!(m.days.len(), 42); // 6 weeks * 7

        // Monday start, Jan 1 2024 is Monday -> first cell is day 1
        assert_eq!(m.days[0], Some(1));
        assert_eq!(m.days[30], Some(31));
        assert_eq!(m.days[31], None);
    }

    #[test]
    fn february_2024_leap_offset() {
        let ctx = base_context();
        let m = MonthData::new(&ctx, 2024, 2);

        // Feb 2024 starts Thursday -> 3 empty cells (Mon, Tue, Wed)
        assert_eq!(m.days[0], None);
        assert_eq!(m.days[1], None);
        assert_eq!(m.days[2], None);
        assert_eq!(m.days[3], Some(1));
        assert_eq!(m.days[31], Some(29));
    }

    #[test]
    fn september_1752_reform_gap() {
        let ctx = base_context();
        let m = MonthData::new(&ctx, 1752, 9);

        assert!(m.days.contains(&Some(1)));
        assert!(m.days.contains(&Some(2)));
        // Days 3-13 should be missing
        for day in 3..=13 {
            assert!(!m.days.contains(&Some(day)), "day {day} should be absent");
        }
        assert!(m.days.contains(&Some(14)));
        assert!(m.days.contains(&Some(30)));
    }

    #[test]
    fn days_and_weekdays_aligned() {
        let ctx = base_context();
        for month in 1..=12 {
            let m = MonthData::new(&ctx, 2024, month);
            for (i, day) in m.days.iter().enumerate() {
                if day.is_some() {
                    assert!(m.weekdays[i].is_some(), "month {month}, idx {i}");
                } else {
                    assert!(m.weekdays[i].is_none(), "month {month}, idx {i}");
                }
            }
        }
    }

    #[test]
    fn sunday_start_offset() {
        let mut ctx = base_context();
        ctx.week_start = Weekday::Sun;
        // Jan 2024: starts Monday. With Sunday start, offset = 1 (Sunday empty)
        let m = MonthData::new(&ctx, 2024, 1);
        assert_eq!(m.days[0], None); // Sunday slot empty
        assert_eq!(m.days[1], Some(1)); // Monday = day 1
    }

    #[test]
    fn week_numbers_when_enabled() {
        let mut ctx = base_context();
        ctx.week_numbers = true;
        let m = MonthData::new(&ctx, 2024, 1);

        // First actual day should have a week number
        let first_day_idx = m.days.iter().position(|d| d.is_some()).unwrap();
        assert!(m.week_numbers[first_day_idx].is_some());
    }
}

// ===========================================================================
// Context creation from Args
// ===========================================================================

mod context_creation {
    use super::*;

    #[test]
    fn default_args() {
        let args = Args::parse_from(["cal"]);
        let ctx = CalContext::new(&args).unwrap();
        assert_eq!(ctx.week_start, Weekday::Mon);
        assert!(!ctx.julian);
        assert!(!ctx.week_numbers);
    }

    #[test]
    fn year_julian_week_numbers() {
        let args = Args::parse_from(["cal", "-y", "-j", "-w"]);
        let ctx = CalContext::new(&args).unwrap();
        assert!(ctx.julian);
        assert!(ctx.week_numbers);
    }

    #[test]
    fn mutually_exclusive_display_modes() {
        // -y and -n conflict
        let args = Args::parse_from(["cal", "-y", "-n", "5"]);
        let err = CalContext::new(&args).unwrap_err();
        assert!(err.contains("mutually exclusive"));
    }

    #[test]
    fn invalid_columns() {
        let args = Args::parse_from(["cal", "-c", "0"]);
        assert!(CalContext::new(&args).is_err());

        let args = Args::parse_from(["cal", "-c", "abc"]);
        assert!(CalContext::new(&args).is_err());
    }

    #[test]
    fn valid_columns() {
        let args = Args::parse_from(["cal", "-c", "4"]);
        let ctx = CalContext::new(&args).unwrap();
        match ctx.columns {
            ColumnsMode::Fixed(n) => assert_eq!(n, 4),
            _ => panic!("expected Fixed columns"),
        }
    }

    #[test]
    fn sunday_start() {
        let args = Args::parse_from(["cal", "-s"]);
        let ctx = CalContext::new(&args).unwrap();
        assert_eq!(ctx.week_start, Weekday::Sun);
    }

    #[test]
    fn color_depends_on_terminal() {
        // Without --color: color = is_terminal (true in tty, false in CI)
        let args = Args::parse_from(["cal"]);
        let ctx = CalContext::new(&args).unwrap();
        assert_eq!(ctx.color, std::io::stdout().is_terminal());

        // With --color: color is always disabled
        let args = Args::parse_from(["cal", "--color"]);
        let ctx = CalContext::new(&args).unwrap();
        assert!(!ctx.color);
    }

    #[test]
    fn reform_gregorian() {
        let args = Args::parse_from(["cal", "--reform", "gregorian"]);
        let ctx = CalContext::new(&args).unwrap();
        assert_eq!(ctx.reform_year, i32::MIN);
    }

    #[test]
    fn reform_julian() {
        let args = Args::parse_from(["cal", "--reform", "julian"]);
        let ctx = CalContext::new(&args).unwrap();
        assert_eq!(ctx.reform_year, i32::MAX);
    }

    #[test]
    fn iso_overrides_reform() {
        let args = Args::parse_from(["cal", "--iso"]);
        let ctx = CalContext::new(&args).unwrap();
        assert_eq!(ctx.reform_year, i32::MIN);
    }

    #[test]
    fn vertical_mode_narrow_gutter() {
        let args = Args::parse_from(["cal", "-v"]);
        let ctx = CalContext::new(&args).unwrap();
        assert!(ctx.vertical);
        assert_eq!(ctx.gutter_width, 1);
    }

    #[test]
    fn span_mode() {
        let args = Args::parse_from(["cal", "-S", "-n", "6"]);
        let ctx = CalContext::new(&args).unwrap();
        assert!(ctx.span);
    }
}

// ===========================================================================
// parse_month
// ===========================================================================

mod parse_month_tests {
    use super::*;

    #[test]
    fn numeric_valid() {
        for n in 1..=12 {
            assert_eq!(parse_month(&n.to_string()), Some(n));
        }
    }

    #[test]
    fn numeric_invalid() {
        assert_eq!(parse_month("0"), None);
        assert_eq!(parse_month("13"), None);
        assert_eq!(parse_month("-1"), None);
        assert_eq!(parse_month("999"), None);
    }

    #[test]
    fn english_full_names() {
        let names = [
            "january",
            "february",
            "march",
            "april",
            "may",
            "june",
            "july",
            "august",
            "september",
            "october",
            "november",
            "december",
        ];
        for (i, name) in names.iter().enumerate() {
            assert_eq!(parse_month(name), Some(i as u32 + 1), "{name}");
        }
    }

    #[test]
    fn english_case_insensitive() {
        assert_eq!(parse_month("January"), Some(1));
        assert_eq!(parse_month("JANUARY"), Some(1));
        assert_eq!(parse_month("jAnUaRy"), Some(1));
    }

    #[test]
    fn english_abbreviations() {
        let abbrevs = [
            ("jan", 1),
            ("feb", 2),
            ("mar", 3),
            ("apr", 4),
            ("jun", 6),
            ("jul", 7),
            ("aug", 8),
            ("sep", 9),
            ("oct", 10),
            ("nov", 11),
            ("dec", 12),
        ];
        for (abbr, expected) in abbrevs {
            assert_eq!(parse_month(abbr), Some(expected), "{abbr}");
        }
    }

    #[test]
    fn russian_names() {
        let names = [
            ("январь", 1),
            ("февраль", 2),
            ("март", 3),
            ("апрель", 4),
            ("май", 5),
            ("июнь", 6),
            ("июль", 7),
            ("август", 8),
            ("сентябрь", 9),
            ("октябрь", 10),
            ("ноябрь", 11),
            ("декабрь", 12),
        ];
        for (name, expected) in names {
            assert_eq!(parse_month(name), Some(expected), "{name}");
        }
    }

    #[test]
    fn garbage_input() {
        assert_eq!(parse_month("abc"), None);
        assert_eq!(parse_month(""), None);
        assert_eq!(parse_month("hello"), None);
    }
}

// ===========================================================================
// get_display_date
// ===========================================================================

mod display_date {
    use super::*;

    #[test]
    fn no_arguments_returns_today() {
        let args = Args::parse_from(["cal"]);
        let (year, month, day) = get_display_date(&args).unwrap();
        let today = chrono::Local::now().date_naive();
        assert_eq!(year, today.year());
        assert_eq!(month, today.month());
        assert_eq!(day, None);
    }

    #[test]
    fn single_arg_four_digit_year() {
        let args = Args::parse_from(["cal", "2026"]);
        let (year, _month, day) = get_display_date(&args).unwrap();
        assert_eq!(year, 2026);
        assert_eq!(day, None);
    }

    #[test]
    fn single_arg_month_number() {
        let args = Args::parse_from(["cal", "2"]);
        let (_year, month, _day) = get_display_date(&args).unwrap();
        assert_eq!(month, 2);
    }

    #[test]
    fn single_arg_month_name() {
        let args = Args::parse_from(["cal", "march"]);
        let (_year, month, _day) = get_display_date(&args).unwrap();
        assert_eq!(month, 3);
    }

    #[test]
    fn two_args_month_year() {
        let args = Args::parse_from(["cal", "2", "2026"]);
        let (year, month, day) = get_display_date(&args).unwrap();
        assert_eq!(year, 2026);
        assert_eq!(month, 2);
        assert_eq!(day, None);
    }

    #[test]
    fn two_args_month_name_year() {
        let args = Args::parse_from(["cal", "february", "2026"]);
        let (year, month, _day) = get_display_date(&args).unwrap();
        assert_eq!(year, 2026);
        assert_eq!(month, 2);
    }

    #[test]
    fn three_args_day_month_year() {
        let args = Args::parse_from(["cal", "15", "3", "2026"]);
        let (year, month, day) = get_display_date(&args).unwrap();
        assert_eq!(year, 2026);
        assert_eq!(month, 3);
        assert_eq!(day, Some(15));
    }

    #[test]
    fn invalid_single_arg() {
        let args = Args::parse_from(["cal", "xyz"]);
        assert!(get_display_date(&args).is_err());
    }

    #[test]
    fn invalid_month_in_two_args() {
        let args = Args::parse_from(["cal", "13", "2026"]);
        assert!(get_display_date(&args).is_err());
    }

    #[test]
    fn invalid_year_range() {
        let args = Args::parse_from(["cal", "1", "0"]);
        assert!(get_display_date(&args).is_err());

        let args = Args::parse_from(["cal", "1", "10000"]);
        assert!(get_display_date(&args).is_err());
    }

    #[test]
    fn invalid_day_range() {
        let args = Args::parse_from(["cal", "0", "1", "2026"]);
        assert!(get_display_date(&args).is_err());

        let args = Args::parse_from(["cal", "32", "1", "2026"]);
        assert!(get_display_date(&args).is_err());
    }
}

// ===========================================================================
// Formatting: headers
// ===========================================================================

mod formatting {
    use super::*;

    #[test]
    fn month_header_with_year() {
        let header = format_month_header(2026, 2, 20, true, false);
        assert!(header.contains("2026"));
        assert_eq!(header.width(), 20);
    }

    #[test]
    fn month_header_without_year() {
        let header = format_month_header(2026, 2, 20, false, false);
        assert!(!header.contains("2026"));
    }

    #[test]
    fn month_header_color_codes() {
        let colored = format_month_header(2026, 2, 20, true, true);
        assert!(colored.starts_with("\x1b[96m"));
        assert!(colored.ends_with("\x1b[0m"));

        let plain = format_month_header(2026, 2, 20, true, false);
        assert!(!plain.contains("\x1b["));
    }

    #[test]
    fn header_width_consistent_across_months() {
        for month in 1..=12 {
            let h = format_month_header(2024, month, 20, true, false);
            assert_eq!(h.width(), 20, "month {month}");
        }
    }

    #[test]
    fn weekday_header_monday_start() {
        let ctx = base_context();
        let header = format_weekday_headers(&ctx, false);
        let mon_pos = header.find("Пн").unwrap();
        let sun_pos = header.find("Вс").unwrap();
        assert!(mon_pos < sun_pos);
    }

    #[test]
    fn weekday_header_sunday_start() {
        let mut ctx = base_context();
        ctx.week_start = Weekday::Sun;
        let header = format_weekday_headers(&ctx, false);
        let sun_pos = header.find("Вс").unwrap();
        let mon_pos = header.find("Пн").unwrap();
        assert!(sun_pos < mon_pos);
    }

    #[test]
    fn weekday_header_color() {
        let mut ctx = base_context();
        ctx.color = true;
        let header = format_weekday_headers(&ctx, false);
        assert!(header.starts_with("\x1b[93m"));
        assert!(header.ends_with("\x1b[0m"));

        ctx.color = false;
        let header = format_weekday_headers(&ctx, false);
        assert!(!header.contains("\x1b["));
    }

    #[test]
    fn weekday_header_julian_mode_has_extra_space() {
        let mut ctx = base_context();
        ctx.julian = true;
        let header = format_weekday_headers(&ctx, false);
        assert!(header.starts_with(' '));
    }

    #[test]
    fn weekday_order_monday_start() {
        let order = get_weekday_order(Weekday::Mon);
        assert_eq!(order[0], Weekday::Mon);
        assert_eq!(order[6], Weekday::Sun);
    }

    #[test]
    fn weekday_order_sunday_start() {
        let order = get_weekday_order(Weekday::Sun);
        assert_eq!(order[0], Weekday::Sun);
        assert_eq!(order[6], Weekday::Sat);
    }
}

// ===========================================================================
// Month grid formatting
// ===========================================================================

mod month_grid {
    use super::*;

    #[test]
    fn grid_structure() {
        let ctx = base_context();
        let m = MonthData::new(&ctx, 2024, 1);
        let grid = format_month_grid(&ctx, &m);

        // Header + weekdays + up to 6 week rows = 8 lines
        assert!(grid.len() >= 8 && grid.len() <= 9);

        // First line: month name header
        assert!(grid[0].contains("Январь"));
        assert!(grid[0].contains("2024"));

        // Second line: weekday names
        assert!(grid[1].contains("Пн"));
    }

    #[test]
    fn grid_contains_all_days() {
        let ctx = base_context();
        let m = MonthData::new(&ctx, 2024, 1);
        let grid = format_month_grid(&ctx, &m);
        let body: String = grid[2..].join("\n");

        assert!(body.contains(" 1"));
        assert!(body.contains("15"));
        assert!(body.contains("31"));
    }

    #[test]
    fn grid_february_leap() {
        let ctx = base_context();
        let m = MonthData::new(&ctx, 2024, 2);
        let grid = format_month_grid(&ctx, &m);
        let body: String = grid[2..].join("\n");
        assert!(body.contains("29"));
    }

    #[test]
    fn grid_february_non_leap() {
        let ctx = base_context();
        let m = MonthData::new(&ctx, 2023, 2);
        let grid = format_month_grid(&ctx, &m);
        let body: String = grid[2..].join("\n");
        assert!(body.contains("28"));
        assert!(!body.contains("29"));
    }

    #[test]
    fn grid_day_rows_consistent_width() {
        let ctx = base_context();
        let m = MonthData::new(&ctx, 2024, 1);
        let grid = format_month_grid(&ctx, &m);

        let expected_width = grid[2].width();
        for (i, line) in grid.iter().enumerate().skip(2) {
            assert_eq!(line.width(), expected_width, "line {i}");
        }
    }

    #[test]
    fn grid_with_week_numbers() {
        let mut ctx = base_context();
        ctx.week_numbers = true;
        let m = MonthData::new(&ctx, 2024, 1);
        let grid = format_month_grid(&ctx, &m);

        // Week number column adds 3 chars, so wider than 20
        assert!(grid[2].width() > 20);
    }

    #[test]
    fn three_months_boundary() {
        let ctx = base_context();
        let prev = MonthData::new(&ctx, 2023, 12);
        let curr = MonthData::new(&ctx, 2024, 1);
        let next = MonthData::new(&ctx, 2024, 2);

        assert_eq!(prev.year, 2023);
        assert_eq!(prev.month, 12);
        assert_eq!(curr.year, 2024);
        assert_eq!(curr.month, 1);
        assert_eq!(next.year, 2024);
        assert_eq!(next.month, 2);
    }
}
