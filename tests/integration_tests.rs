//! Integration tests for calendar calculation logic.

use chrono::Weekday;
use unicode_width::UnicodeWidthStr;

use cal::formatter::parse_month;
use cal::types::{CalContext, ColumnsMode, MonthData, ReformType, WeekType};

fn test_context() -> CalContext {
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
        ..test_context()
    }
}

fn gregorian_context() -> CalContext {
    CalContext {
        reform_year: ReformType::Gregorian.reform_year(),
        ..test_context()
    }
}

mod leap_year_tests {
    use super::*;

    #[test]
    fn test_gregorian_leap_year_divisible_by_400() {
        let ctx = gregorian_context();
        assert!(ctx.is_leap_year(2000));
        assert!(ctx.is_leap_year(2400));
    }

    #[test]
    fn test_gregorian_leap_year_divisible_by_4_not_100() {
        let ctx = gregorian_context();
        assert!(ctx.is_leap_year(2024));
        assert!(ctx.is_leap_year(2028));
        assert!(!ctx.is_leap_year(2023));
        assert!(!ctx.is_leap_year(2025));
    }

    #[test]
    fn test_gregorian_not_leap_year_divisible_by_100() {
        let ctx = gregorian_context();
        assert!(!ctx.is_leap_year(1900));
        assert!(!ctx.is_leap_year(2100));
    }

    #[test]
    fn test_julian_leap_year() {
        let ctx = julian_context();
        assert!(ctx.is_leap_year(2024));
        assert!(ctx.is_leap_year(2028));
        assert!(ctx.is_leap_year(1900));
        assert!(!ctx.is_leap_year(2023));
    }

    #[test]
    fn test_year_1752_leap_year() {
        let ctx = test_context();
        assert!(ctx.is_leap_year(1752));
    }
}

mod days_in_month_tests {
    use super::*;

    #[test]
    fn test_31_day_months() {
        let ctx = test_context();
        for month in [1, 3, 5, 7, 8, 10, 12] {
            assert_eq!(ctx.days_in_month(2024, month), 31);
        }
    }

    #[test]
    fn test_30_day_months() {
        let ctx = test_context();
        for month in [4, 6, 9, 11] {
            assert_eq!(ctx.days_in_month(2024, month), 30);
        }
    }

    #[test]
    fn test_february_leap_year() {
        let ctx = test_context();
        assert_eq!(ctx.days_in_month(2024, 2), 29);
        assert_eq!(ctx.days_in_month(2028, 2), 29);
    }

    #[test]
    fn test_february_non_leap_year() {
        let ctx = test_context();
        assert_eq!(ctx.days_in_month(2023, 2), 28);
        assert_eq!(ctx.days_in_month(2025, 2), 28);
    }
}

mod first_day_tests {
    use super::*;

    #[test]
    fn test_first_day_known_dates() {
        let ctx = test_context();

        assert_eq!(ctx.first_day_of_month(2024, 1), Weekday::Mon);
        assert_eq!(ctx.first_day_of_month(2025, 1), Weekday::Wed);
        assert_eq!(ctx.first_day_of_month(2024, 2), Weekday::Thu);
    }

    #[test]
    fn test_first_day_september_1752() {
        let ctx = test_context();
        assert_eq!(ctx.first_day_of_month(1752, 9), Weekday::Fri);
    }
}

mod reform_gap_tests {
    use super::*;

    #[test]
    fn test_reform_gap_detection() {
        let ctx = test_context();

        assert!(ctx.is_reform_gap(1752, 9, 3));
        assert!(ctx.is_reform_gap(1752, 9, 13));
        assert!(ctx.is_reform_gap(1752, 9, 8));

        assert!(!ctx.is_reform_gap(1752, 9, 2));
        assert!(!ctx.is_reform_gap(1752, 9, 14));

        assert!(!ctx.is_reform_gap(1752, 8, 5));
        assert!(!ctx.is_reform_gap(1752, 10, 5));
        assert!(!ctx.is_reform_gap(1751, 9, 5));
    }

    #[test]
    fn test_no_reform_gap_gregorian() {
        let ctx = gregorian_context();
        assert!(!ctx.is_reform_gap(1752, 9, 5));
    }

    #[test]
    fn test_no_reform_gap_julian() {
        let ctx = julian_context();
        assert!(!ctx.is_reform_gap(1752, 9, 5));
    }
}

mod day_of_year_tests {
    use super::*;

    #[test]
    fn test_day_of_year_non_leap() {
        let ctx = test_context();

        assert_eq!(ctx.day_of_year(2023, 1, 1), 1);
        assert_eq!(ctx.day_of_year(2023, 1, 31), 31);
        assert_eq!(ctx.day_of_year(2023, 2, 1), 32);
        assert_eq!(ctx.day_of_year(2023, 12, 31), 365);
    }

    #[test]
    fn test_day_of_year_leap() {
        let ctx = test_context();

        assert_eq!(ctx.day_of_year(2024, 1, 1), 1);
        assert_eq!(ctx.day_of_year(2024, 2, 29), 60);
        assert_eq!(ctx.day_of_year(2024, 3, 1), 61);
        assert_eq!(ctx.day_of_year(2024, 12, 31), 366);
    }

    #[test]
    fn test_day_of_year_reform_gap() {
        let ctx = test_context();

        assert_eq!(ctx.day_of_year(1752, 9, 2), 235);
        assert_eq!(ctx.day_of_year(1752, 9, 14), 247);
    }
}

mod month_data_tests {
    use super::*;

    #[test]
    fn test_month_data_january_2024() {
        let ctx = test_context();
        let month = MonthData::new(&ctx, 2024, 1);

        assert_eq!(month.year, 2024);
        assert_eq!(month.month, 1);
        assert_eq!(month.days.len(), 42);

        // January 2024 starts on Monday
        assert_eq!(month.days[0], Some(1));
        assert_eq!(month.days[30], Some(31));
        assert_eq!(month.days[31], None);
    }

    #[test]
    fn test_month_data_february_2024_leap() {
        let ctx = test_context();
        let month = MonthData::new(&ctx, 2024, 2);

        // February 2024 starts on Thursday
        assert_eq!(month.days[0], None);
        assert_eq!(month.days[1], None);
        assert_eq!(month.days[2], None);
        assert_eq!(month.days[3], Some(1));
        assert_eq!(month.days[31], Some(29));
    }

    #[test]
    fn test_month_data_september_1752_reform() {
        let ctx = test_context();
        let month = MonthData::new(&ctx, 1752, 9);

        assert!(month.days.contains(&Some(2)));
        assert!(!month.days.contains(&Some(3)));
        assert!(!month.days.contains(&Some(13)));
        assert!(month.days.contains(&Some(14)));
    }

    #[test]
    fn test_month_data_weekday_alignment() {
        let ctx = test_context();

        for month in 1..=12 {
            let month_data = MonthData::new(&ctx, 2024, month);

            for (i, day) in month_data.days.iter().enumerate() {
                if day.is_some() {
                    assert!(month_data.weekdays[i].is_some());
                } else {
                    assert!(month_data.weekdays[i].is_none());
                }
            }
        }
    }
}

mod weekend_tests {
    use super::*;

    #[test]
    fn test_is_weekend() {
        let ctx = test_context();

        assert!(ctx.is_weekend(Weekday::Sat));
        assert!(ctx.is_weekend(Weekday::Sun));
        assert!(!ctx.is_weekend(Weekday::Mon));
        assert!(!ctx.is_weekend(Weekday::Tue));
        assert!(!ctx.is_weekend(Weekday::Wed));
        assert!(!ctx.is_weekend(Weekday::Thu));
        assert!(!ctx.is_weekend(Weekday::Fri));
    }
}

mod week_number_tests {
    use super::*;

    #[test]
    fn test_iso_week_number() {
        let mut ctx = test_context();
        ctx.week_type = WeekType::Iso;

        assert_eq!(ctx.week_number(2024, 1, 1), 1);

        let week = ctx.week_number(2024, 12, 30);
        assert!(week == 1 || week == 53);
    }

    #[test]
    fn test_us_week_number() {
        let mut ctx = test_context();
        ctx.week_type = WeekType::Us;

        assert_eq!(ctx.week_number(2024, 1, 1), 1);
    }
}

mod context_validation_tests {
    use super::*;
    use cal::args::Args;
    use clap::Parser;

    #[test]
    fn test_context_creation_default() {
        let args = Args::parse_from(["cal"]);
        let ctx = CalContext::new(&args);
        assert!(ctx.is_ok());
    }

    #[test]
    fn test_context_creation_with_options() {
        let args = Args::parse_from(["cal", "-y", "-j", "-w"]);
        let ctx = CalContext::new(&args);
        assert!(ctx.is_ok());
        let ctx = ctx.unwrap();
        assert!(ctx.julian);
        assert!(ctx.week_numbers);
    }

    #[test]
    fn test_context_mutually_exclusive_options() {
        let args = Args::parse_from(["cal", "-y", "-n", "5"]);
        let result = CalContext::new(&args);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("mutually exclusive"));
    }

    #[test]
    fn test_context_invalid_columns() {
        let args = Args::parse_from(["cal", "-c", "0"]);
        assert!(CalContext::new(&args).is_err());

        let args = Args::parse_from(["cal", "-c", "abc"]);
        assert!(CalContext::new(&args).is_err());
    }

    #[test]
    fn test_context_sunday_start() {
        let args = Args::parse_from(["cal", "-s"]);
        let ctx = CalContext::new(&args).unwrap();
        assert_eq!(ctx.week_start, Weekday::Sun);
    }

    #[test]
    fn test_context_color_settings() {
        let args = Args::parse_from(["cal"]);
        let ctx = CalContext::new(&args).unwrap();
        assert!(!ctx.color);

        let args = Args::parse_from(["cal", "--color"]);
        let ctx = CalContext::new(&args).unwrap();
        assert!(!ctx.color);
    }

    #[test]
    fn test_context_reform_types() {
        let args = Args::parse_from(["cal", "--reform", "gregorian"]);
        let ctx = CalContext::new(&args).unwrap();
        assert_eq!(ctx.reform_year, i32::MIN);

        let args = Args::parse_from(["cal", "--reform", "julian"]);
        let ctx = CalContext::new(&args).unwrap();
        assert_eq!(ctx.reform_year, i32::MAX);

        let args = Args::parse_from(["cal", "--iso"]);
        let ctx = CalContext::new(&args).unwrap();
        assert_eq!(ctx.reform_year, i32::MIN);
    }
}

mod parse_month_tests {
    use super::*;

    #[test]
    fn test_parse_month_numeric() {
        for (input, expected) in [
            ("1", Some(1)),
            ("2", Some(2)),
            ("12", Some(12)),
            ("0", None),
            ("13", None),
            ("abc", None),
        ] {
            assert_eq!(parse_month(input), expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_parse_month_english_names() {
        assert_eq!(parse_month("january"), Some(1));
        assert_eq!(parse_month("January"), Some(1));
        assert_eq!(parse_month("JANUARY"), Some(1));
        assert_eq!(parse_month("february"), Some(2));
        assert_eq!(parse_month("december"), Some(12));
    }

    #[test]
    fn test_parse_month_english_short() {
        assert_eq!(parse_month("jan"), Some(1));
        assert_eq!(parse_month("feb"), Some(2));
        assert_eq!(parse_month("mar"), Some(3));
        assert_eq!(parse_month("apr"), Some(4));
        assert_eq!(parse_month("jun"), Some(6));
        assert_eq!(parse_month("jul"), Some(7));
        assert_eq!(parse_month("aug"), Some(8));
        assert_eq!(parse_month("sep"), Some(9));
        assert_eq!(parse_month("oct"), Some(10));
        assert_eq!(parse_month("nov"), Some(11));
        assert_eq!(parse_month("dec"), Some(12));
    }

    #[test]
    fn test_parse_month_russian() {
        assert_eq!(parse_month("январь"), Some(1));
        assert_eq!(parse_month("февраль"), Some(2));
        assert_eq!(parse_month("декабрь"), Some(12));
    }
}

mod layout_tests {
    use super::*;
    use cal::formatter::{
        format_month_grid, format_month_header, format_weekday_headers, get_weekday_order,
    };

    #[test]
    fn test_month_header_format() {
        let header = format_month_header(2026, 2, 20, true, false);
        assert!(header.contains("Февраль"));
        assert!(header.contains("2026"));
        assert!(header.width() >= 20);
    }

    #[test]
    fn test_month_header_without_year() {
        let header = format_month_header(2026, 2, 20, false, false);
        assert!(header.contains("Февраль"));
        assert!(!header.contains("2026"));
    }

    #[test]
    fn test_month_header_with_color() {
        let header = format_month_header(2026, 2, 20, true, true);
        assert!(header.contains("\x1b[96m"));
        assert!(header.contains("\x1b[0m"));
    }

    #[test]
    fn test_weekday_header_structure_monday_start() {
        let ctx = test_context();
        let header = format_weekday_headers(&ctx, false);

        assert!(header.contains("Пн"));
        assert!(header.contains("Вт"));
        assert!(header.contains("Ср"));
        assert!(header.contains("Чт"));
        assert!(header.contains("Пт"));
        assert!(header.contains("Сб"));
        assert!(header.contains("Вс"));

        let mon_pos = header.find("Пн").unwrap();
        let tue_pos = header.find("Вт").unwrap();
        assert!(mon_pos < tue_pos);
    }

    #[test]
    fn test_weekday_header_structure_sunday_start() {
        let mut ctx = test_context();
        ctx.week_start = chrono::Weekday::Sun;
        let header = format_weekday_headers(&ctx, false);

        let sun_pos = header.find("Вс").unwrap();
        let mon_pos = header.find("Пн").unwrap();
        assert!(sun_pos < mon_pos);
    }

    #[test]
    fn test_weekday_header_with_week_numbers() {
        let mut ctx = test_context();
        ctx.week_numbers = true;
        let header = format_weekday_headers(&ctx, false);
        assert!(header.len() > 20);
    }

    #[test]
    fn test_weekday_header_with_julian() {
        let mut ctx = test_context();
        ctx.julian = true;
        let header = format_weekday_headers(&ctx, false);
        assert!(header.starts_with(" "));
    }

    #[test]
    fn test_month_grid_line_count() {
        let ctx = test_context();
        let month = MonthData::new(&ctx, 2024, 1);
        let grid = format_month_grid(&ctx, &month);

        assert!(grid.len() >= 8);
        assert!(grid.len() <= 9);
    }

    #[test]
    fn test_month_grid_first_line_is_header() {
        let ctx = test_context();
        let month = MonthData::new(&ctx, 2024, 1);
        let grid = format_month_grid(&ctx, &month);

        assert!(grid[0].contains("Январь"));
        assert!(grid[0].contains("2024"));
    }

    #[test]
    fn test_month_grid_second_line_is_weekdays() {
        let ctx = test_context();
        let month = MonthData::new(&ctx, 2024, 1);
        let grid = format_month_grid(&ctx, &month);

        assert!(grid[1].contains("Пн"));
    }

    #[test]
    fn test_month_grid_contains_day_1() {
        let ctx = test_context();
        let month = MonthData::new(&ctx, 2024, 1);
        let grid = format_month_grid(&ctx, &month);

        let days_area: String = grid[2..].join("\n");
        assert!(days_area.contains(" 1"));
    }

    #[test]
    fn test_month_grid_contains_last_day() {
        let ctx = test_context();
        let month = MonthData::new(&ctx, 2024, 1);
        let grid = format_month_grid(&ctx, &month);

        let days_area: String = grid[2..].join("\n");
        assert!(days_area.contains("31"));
    }

    #[test]
    fn test_month_grid_february_leap_year() {
        let ctx = test_context();
        let month = MonthData::new(&ctx, 2024, 2);
        let grid = format_month_grid(&ctx, &month);

        let days_area: String = grid[2..].join("\n");
        assert!(days_area.contains("29"));
    }

    #[test]
    fn test_month_grid_february_non_leap_year() {
        let ctx = test_context();
        let month = MonthData::new(&ctx, 2023, 2);
        let grid = format_month_grid(&ctx, &month);

        let days_area: String = grid[2..].join("\n");
        assert!(days_area.contains("28"));
        assert!(!days_area.contains("29"));
    }

    #[test]
    fn test_vertical_layout_weekday_order() {
        let ctx = test_context();
        let weekday_order = get_weekday_order(ctx.week_start);

        assert_eq!(weekday_order[0], chrono::Weekday::Mon);
        assert_eq!(weekday_order[6], chrono::Weekday::Sun);
    }

    #[test]
    fn test_vertical_layout_days_in_columns() {
        let ctx = test_context();
        let month = MonthData::new(&ctx, 2024, 1);

        let day1_pos = month.days.iter().position(|&d| d == Some(1)).unwrap();
        let day8_pos = month.days.iter().position(|&d| d == Some(8)).unwrap();

        assert_eq!(day8_pos - day1_pos, 7);
    }

    #[test]
    fn test_three_months_structure() {
        let ctx = test_context();
        let prev = MonthData::new(&ctx, 2024, 1);
        let curr = MonthData::new(&ctx, 2024, 2);
        let next = MonthData::new(&ctx, 2024, 3);

        assert_eq!(prev.month, 1);
        assert_eq!(curr.month, 2);
        assert_eq!(next.month, 3);
    }

    #[test]
    fn test_three_months_year_boundary() {
        let ctx = test_context();

        let prev = MonthData::new(&ctx, 2023, 12);
        let curr = MonthData::new(&ctx, 2024, 1);
        let next = MonthData::new(&ctx, 2024, 2);

        assert_eq!(prev.year, 2023);
        assert_eq!(curr.year, 2024);
        assert_eq!(next.year, 2024);
    }

    #[test]
    fn test_color_codes_in_header() {
        let header = format_month_header(2024, 1, 20, true, true);
        assert!(header.starts_with("\x1b[96m"));
        assert!(header.ends_with("\x1b[0m"));
    }

    #[test]
    fn test_no_color_codes_when_disabled() {
        let header = format_month_header(2024, 1, 20, true, false);
        assert!(!header.contains("\x1b[96m"));
        assert!(!header.contains("\x1b[0m"));
    }

    #[test]
    fn test_weekday_header_color_placement() {
        let mut ctx = test_context();
        ctx.color = true;
        let header = format_weekday_headers(&ctx, false);

        assert!(header.starts_with("\x1b[93m"));
        assert!(header.ends_with("\x1b[0m"));
    }

    #[test]
    fn test_weekday_header_no_color_when_disabled() {
        let mut ctx = test_context();
        ctx.color = false;
        let header = format_weekday_headers(&ctx, false);

        assert!(!header.contains("\x1b[93m"));
        assert!(!header.contains("\x1b[0m"));
    }

    #[test]
    fn test_header_width_consistency() {
        let width = 20;
        let header1 = format_month_header(2024, 1, width, true, false);
        let header2 = format_month_header(2024, 12, width, true, false);

        assert_eq!(header1.width(), width);
        assert_eq!(header2.width(), width);
    }

    #[test]
    fn test_day_alignment_in_grid() {
        let ctx = test_context();
        let month = MonthData::new(&ctx, 2024, 1);
        let grid = format_month_grid(&ctx, &month);

        let expected_width = grid[2].width();
        for (i, line) in grid.iter().enumerate().skip(2) {
            assert_eq!(
                line.width(),
                expected_width,
                "Line {} has inconsistent width",
                i
            );
        }
    }
}

mod get_display_date_tests {
    use cal::args::{Args, get_display_date};
    use clap::Parser;

    #[test]
    fn test_single_year_argument() {
        let args = Args::parse_from(["cal", "2026"]);
        let (year, _month, day) = get_display_date(&args).unwrap();
        assert_eq!(year, 2026);
        assert_eq!(day, None);
    }

    #[test]
    fn test_single_month_argument() {
        let args = Args::parse_from(["cal", "2"]);
        let (_year, month, _day) = get_display_date(&args).unwrap();
        assert_eq!(month, 2);
    }

    #[test]
    fn test_month_year_arguments() {
        let args = Args::parse_from(["cal", "2", "2026"]);
        let (year, month, _day) = get_display_date(&args).unwrap();
        assert_eq!(year, 2026);
        assert_eq!(month, 2);
    }
}
