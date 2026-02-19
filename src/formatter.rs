//! Calendar formatting and display with localization and color support.

use chrono::{Datelike, Locale, NaiveDate, Weekday};
use unicode_width::UnicodeWidthStr;

use crate::types::{
    COLOR_RED, COLOR_RESET, COLOR_REVERSE, COLOR_SAND_YELLOW, COLOR_TEAL, CalContext,
    GUTTER_WIDTH_YEAR, MonthData,
};

#[cfg(feature = "plugins")]
use std::sync::Mutex;

#[cfg(feature = "plugins")]
static PLUGIN: Mutex<Option<crate::plugin_api::PluginHandle>> = Mutex::new(None);

#[cfg(feature = "plugins")]
static COUNTRY: Mutex<Option<String>> = Mutex::new(None);

/// Cache for holiday data: (year, month, data) or (year, 0, full_year_data)
#[cfg(feature = "plugins")]
static HOLIDAY_CACHE: Mutex<Option<(i32, u32, String)>> = Mutex::new(None);

/// Preload holiday data for entire year (called when -y flag is used)
#[cfg(feature = "plugins")]
pub fn preload_year_holidays(ctx: &CalContext, year: i32) {
    if !ctx.holidays {
        return;
    }

    // Check if already cached
    {
        let cache_guard = HOLIDAY_CACHE.lock().unwrap();
        if let Some((cached_year, cached_month, _)) = &*cache_guard
            && *cached_year == year
            && *cached_month == 0
        {
            return;
        }
    }

    if !init_plugin() {
        return;
    }

    let plugin_guard = PLUGIN.lock().unwrap();
    let country_guard = COUNTRY.lock().unwrap();

    if let (Some(plugin), Some(country)) = (&*plugin_guard, &*country_guard)
        && let Some(data) = plugin.get_year_holidays(year, country)
    {
        let mut cache_guard = HOLIDAY_CACHE.lock().unwrap();
        *cache_guard = Some((year, 0, data));
    }
}

#[cfg(not(feature = "plugins"))]
pub fn preload_year_holidays(_ctx: &CalContext, _year: i32) {}

#[cfg(feature = "plugins")]
fn init_plugin() -> bool {
    let mut plugin_guard = PLUGIN.lock().unwrap();
    if plugin_guard.is_some() {
        return true;
    }

    if let Some(plugin) = crate::plugin_api::try_load_plugin() {
        let country = plugin.get_country_from_locale();
        let mut country_guard = COUNTRY.lock().unwrap();
        *country_guard = Some(country.clone());
        *plugin_guard = Some(plugin);
        true
    } else {
        false
    }
}

#[cfg(feature = "plugins")]
fn get_holiday_code(ctx: &CalContext, year: i32, month: u32, day: u32) -> i32 {
    if !ctx.holidays {
        return 0;
    }

    // Check cache (including full year cache with month=0)
    {
        let cache_guard = HOLIDAY_CACHE.lock().unwrap();
        if let Some((cached_year, cached_month, data)) = &*cache_guard
            && *cached_year == year
        {
            let day_idx = if *cached_month == 0 {
                // Full year data - calculate day of year
                let date = chrono::NaiveDate::from_ymd_opt(year, month, day);
                if let Some(d) = date {
                    d.ordinal() as usize - 1
                } else {
                    return 0;
                }
            } else if *cached_month == month {
                // Month data
                (day - 1) as usize
            } else {
                return 0;
            };

            if day_idx < data.len() {
                return data
                    .chars()
                    .nth(day_idx)
                    .and_then(|c| c.to_digit(10).map(|d| d as i32))
                    .unwrap_or(0);
            }
        }
    }

    // Cache miss - fetch data for the month
    if !init_plugin() {
        return 0;
    }

    let plugin_guard = PLUGIN.lock().unwrap();
    let country_guard = COUNTRY.lock().unwrap();

    if let (Some(plugin), Some(country)) = (&*plugin_guard, &*country_guard)
        && let Some(data) = plugin.get_holidays(year, month, country)
    {
        // Don't overwrite full year cache with month data
        let mut cache_guard = HOLIDAY_CACHE.lock().unwrap();
        let should_update = match &*cache_guard {
            Some((cached_year, cached_month, _)) => !(*cached_year == year && *cached_month == 0),
            None => true,
        };

        if should_update {
            *cache_guard = Some((year, month, data.clone()));
        }

        let day_idx = (day - 1) as usize;
        if day_idx < data.len() {
            return data
                .chars()
                .nth(day_idx)
                .and_then(|c| c.to_digit(10).map(|d| d as i32))
                .unwrap_or(0);
        }
    }

    0
}

#[cfg(not(feature = "plugins"))]
fn get_holiday_code(_ctx: &CalContext, _year: i32, _month: u32, _day: u32) -> i32 {
    0
}

#[cfg(feature = "plugins")]
pub fn preload_holidays(ctx: &CalContext, year: i32, month: u32) {
    if !ctx.holidays {
        return;
    }

    {
        let cache_guard = HOLIDAY_CACHE.lock().unwrap();
        if let Some((cached_year, cached_month, _)) = &*cache_guard
            && *cached_year == year
            && (*cached_month == month || *cached_month == 0)
        {
            return;
        }
    }

    let _ = get_holiday_code(ctx, year, month, 1);
}

#[cfg(not(feature = "plugins"))]
pub fn preload_holidays(_ctx: &CalContext, _year: i32, _month: u32) {}

/// Get system locale from environment (LC_ALL > LC_TIME > LANG > en_US).
pub fn get_system_locale() -> Locale {
    std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LC_TIME"))
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_else(|_| "en_US.UTF-8".to_string())
        .split('.')
        .next()
        .unwrap_or("en_US")
        .split('@')
        .next()
        .unwrap_or("en_US")
        .parse()
        .unwrap_or(Locale::en_US)
}

/// Get month name in nominative case for current locale.
pub fn get_month_name(month: u32) -> String {
    let locale = get_system_locale();

    match locale {
        Locale::ru_RU => [
            "Январь",
            "Февраль",
            "Март",
            "Апрель",
            "Май",
            "Июнь",
            "Июль",
            "Август",
            "Сентябрь",
            "Октябрь",
            "Ноябрь",
            "Декабрь",
        ][(month - 1) as usize]
            .to_string(),
        Locale::uk_UA => [
            "Січень",
            "Лютий",
            "Березень",
            "Квітень",
            "Травень",
            "Червень",
            "Липень",
            "Серпень",
            "Вересень",
            "Жовтень",
            "Листопад",
            "Грудень",
        ][(month - 1) as usize]
            .to_string(),
        Locale::be_BY => [
            "Студзень",
            "Люты",
            "Сакавік",
            "Красавік",
            "Май",
            "Чэрвень",
            "Ліпень",
            "Жнівень",
            "Верасень",
            "Кастрычнік",
            "Лістапад",
            "Снежань",
        ][(month - 1) as usize]
            .to_string(),
        _ => {
            let date = NaiveDate::from_ymd_opt(2000, month, 1).unwrap();
            date.format_localized("%B", locale).to_string()
        }
    }
}

/// Parse month from string (numeric 1-12 or name in English/Russian).
pub fn parse_month(s: &str) -> Option<u32> {
    if let Ok(n) = s.parse::<u32>()
        && (1..=12).contains(&n)
    {
        return Some(n);
    }

    let s_lower = s.to_lowercase();
    let month_names: [(&str, u32); 35] = [
        // English full names
        ("january", 1),
        ("february", 2),
        ("march", 3),
        ("april", 4),
        ("may", 5),
        ("june", 6),
        ("july", 7),
        ("august", 8),
        ("september", 9),
        ("october", 10),
        ("november", 11),
        ("december", 12),
        // Russian full names
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
        // English short forms
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
    month_names
        .iter()
        .find(|(name, _)| *name == s_lower)
        .map(|(_, num)| *num)
}

/// Format month header with optional year and color.
pub fn format_month_header(
    year: i32,
    month: u32,
    width: usize,
    show_year: bool,
    color: bool,
) -> String {
    let month_name = get_month_name(month);
    let header = if show_year {
        format!("{} {}", month_name, year)
    } else {
        month_name
    };
    let centered = center_text(&header, width);
    if color {
        format!("{}{}{}", COLOR_TEAL, centered, COLOR_RESET)
    } else {
        centered
    }
}

/// Center text within a specified width, accounting for Unicode character widths.
fn center_text(text: &str, width: usize) -> String {
    let text_width = text.width();
    if text_width >= width {
        return text.to_string();
    }
    let total_padding = width - text_width;
    let left_padding = total_padding.div_ceil(2);
    let right_padding = total_padding - left_padding;
    format!(
        "{}{}{}",
        " ".repeat(left_padding),
        text,
        " ".repeat(right_padding)
    )
}

/// Get weekday order based on week start day.
pub fn get_weekday_order(week_start: Weekday) -> [Weekday; 7] {
    match week_start {
        Weekday::Mon => [
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
            Weekday::Sat,
            Weekday::Sun,
        ],
        Weekday::Sun => [
            Weekday::Sun,
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
            Weekday::Sat,
        ],
        _ => unreachable!(),
    }
}

/// Get 2-character weekday abbreviation for current locale.
pub fn get_weekday_short_name(weekday: Weekday, locale: Locale) -> String {
    let base_date = NaiveDate::from_ymd_opt(2000, 1, 3).unwrap();
    let offset = weekday.num_days_from_monday() as i64;
    let date = base_date + chrono::Duration::days(offset);
    let day_name = date.format_localized("%a", locale).to_string();
    day_name.chars().take(2).collect()
}

/// Format weekday header row with optional week numbers and color.
pub fn format_weekday_headers(ctx: &CalContext, week_numbers: bool) -> String {
    let locale = get_system_locale();
    let mut result = String::new();

    if week_numbers {
        result.push_str("   ");
    }

    if ctx.julian {
        result.push(' ');
    }

    let weekday_order = get_weekday_order(ctx.week_start);

    if ctx.color {
        result.push_str(COLOR_SAND_YELLOW);
    }

    for (i, &weekday) in weekday_order.iter().enumerate() {
        let short_name = get_weekday_short_name(weekday, locale);

        if ctx.julian {
            if i < 6 {
                result.push_str(&format!("{}  ", short_name));
            } else {
                result.push_str(&format!(" {}", short_name));
            }
        } else if i < 6 {
            result.push_str(&format!("{} ", short_name));
        } else {
            result.push_str(&short_name);
        }
    }

    if ctx.color {
        result.push_str(COLOR_RESET);
    }

    result
}

/// Format day cell with color highlighting.
///
/// Color priority: today > shortened day > weekend/holiday > regular
fn format_day(
    ctx: &CalContext,
    day: u32,
    month: u32,
    year: i32,
    weekday: Weekday,
    is_last: bool,
) -> String {
    let is_today = ctx.color
        && ctx.today.day() == day
        && ctx.today.month() == month
        && ctx.today.year() == year;

    let is_weekend = ctx.color && ctx.is_weekend(weekday);
    let holiday_code = if ctx.color {
        get_holiday_code(ctx, year, month, day)
    } else {
        0
    };
    let day_str = format!("{:>2}", day);

    let formatted = if is_today {
        format!("{}{}{}", COLOR_REVERSE, day_str, COLOR_RESET)
    } else if holiday_code == 2 {
        format!("{}{}{}", COLOR_TEAL, day_str, COLOR_RESET)
    } else if is_weekend || holiday_code == 1 || holiday_code == 8 {
        format!("{}{}{}", COLOR_RED, day_str, COLOR_RESET)
    } else {
        day_str
    };

    if is_last {
        formatted
    } else {
        format!("{} ", formatted)
    }
}

/// Format month as grid of lines (horizontal layout).
pub fn format_month_grid(ctx: &CalContext, month: &MonthData) -> Vec<String> {
    let mut lines = Vec::with_capacity(8);

    let header_width = if ctx.julian {
        27
    } else if ctx.week_numbers {
        23
    } else {
        20
    };

    let month_header = format_month_header(
        month.year,
        month.month,
        header_width,
        ctx.show_year_in_header,
        ctx.color,
    );
    lines.push(month_header);

    let weekday_header = format_weekday_headers(ctx, ctx.week_numbers);
    lines.push(weekday_header);

    let mut day_idx = 0;
    let total_days = month.days.len();

    // Generate 6 weeks of calendar
    for _week in 0..6 {
        let mut line = String::new();

        if ctx.week_numbers {
            let week_wn = (0..7)
                .filter_map(|d| {
                    let idx = day_idx + d;
                    if idx < total_days {
                        month.week_numbers.get(idx).copied().flatten()
                    } else {
                        None
                    }
                })
                .next();

            if let Some(wn) = week_wn {
                line.push_str(&format!("{:>2} ", wn));
            } else {
                line.push_str("   ");
            }
        }

        for day_in_week in 0..7 {
            if day_idx >= total_days {
                break;
            }
            let is_last = (day_in_week + 1) % 7 == 0;

            if let Some(day) = month.days[day_idx] {
                if ctx.julian {
                    let doy = ctx.day_of_year(month.year, month.month, day);
                    let doy_str = format!("{:>3}", doy);
                    if is_last {
                        line.push_str(&doy_str);
                    } else {
                        line.push_str(&format!("{} ", doy_str));
                    }
                } else {
                    let weekday = month.weekdays[day_idx].unwrap();
                    line.push_str(&format_day(
                        ctx,
                        day,
                        month.month,
                        month.year,
                        weekday,
                        is_last,
                    ));
                }
            } else if ctx.julian {
                if is_last {
                    line.push_str("   ");
                } else {
                    line.push_str("    ");
                }
            } else if is_last {
                line.push_str("  ");
            } else {
                line.push_str("   ");
            }
            day_idx += 1;
        }

        lines.push(line);

        if day_idx >= total_days {
            break;
        }
    }

    lines
}

/// Print single month in horizontal (default) or vertical layout.
pub fn print_month(ctx: &CalContext, year: i32, month: u32) {
    preload_holidays(ctx, year, month);

    let month_data = MonthData::new(ctx, year, month);
    if ctx.vertical {
        print_month_vertical(ctx, &month_data, true);
    } else {
        let lines = format_month_grid(ctx, &month_data);
        for line in lines {
            println!("{}", line);
        }
    }
}

/// Print single month in vertical layout (days in columns).
pub fn print_month_vertical(ctx: &CalContext, month: &MonthData, is_first: bool) {
    let month_name = get_month_name(month.month);
    let header = if ctx.show_year_in_header {
        format!("{} {}", month_name, month.year)
    } else {
        month_name.to_string()
    };
    let month_width = 18;

    let padded_header = if is_first {
        format!(
            "    {:<width$}{}",
            header,
            " ".repeat(ctx.gutter_width),
            width = month_width
        )
    } else {
        format!(
            "{:<width$}{}",
            header,
            " ".repeat(ctx.gutter_width),
            width = month_width
        )
    };

    if ctx.color {
        println!("{}{}{}", COLOR_TEAL, padded_header, COLOR_RESET);
    } else {
        println!("{}", padded_header);
    }

    let locale = get_system_locale();
    let weekday_order = get_weekday_order(ctx.week_start);
    let weekday_names: Vec<String> = weekday_order
        .iter()
        .map(|&w| get_weekday_short_name(w, locale))
        .collect();

    for (row, weekday) in weekday_order.iter().enumerate() {
        let day_short = &weekday_names[row];
        if ctx.color {
            print!("{}{}{}", COLOR_SAND_YELLOW, day_short, COLOR_RESET);
        } else {
            print!("{}", day_short);
        }

        for week in 0..6 {
            let day_idx = (*weekday as usize) + 7 * week;
            if day_idx < month.days.len() {
                if let Some(day) = month.days[day_idx] {
                    print_day_vertical(ctx, day, month, *weekday);
                } else {
                    print!("   ");
                }
            }
        }
        println!();
    }
}

/// Print day cell in vertical layout with color highlighting.
fn print_day_vertical(ctx: &CalContext, day: u32, month: &MonthData, weekday: Weekday) {
    let is_today = ctx.color
        && ctx.today.day() == day
        && ctx.today.month() == month.month
        && ctx.today.year() == month.year;

    let is_weekend = ctx.color && ctx.is_weekend(weekday);
    let holiday_code = if ctx.color {
        get_holiday_code(ctx, month.year, month.month, day)
    } else {
        0
    };
    let day_str = day.to_string();
    let padding = 3 - day_str.len();

    let formatted = if is_today {
        format!(
            "{}{}{}{}",
            " ".repeat(padding),
            COLOR_REVERSE,
            day,
            COLOR_RESET
        )
    } else if holiday_code == 2 {
        format!(
            "{}{}{}{}",
            " ".repeat(padding),
            COLOR_TEAL,
            day,
            COLOR_RESET
        )
    } else if is_weekend || holiday_code == 1 || holiday_code == 8 {
        format!("{}{}{}{}", " ".repeat(padding), COLOR_RED, day, COLOR_RESET)
    } else {
        format!("{:>3}", day)
    };
    print!("{}", formatted);
}

/// Print three months side by side (prev, current, next).
pub fn print_three_months(ctx: &CalContext, year: i32, month: u32) {
    let prev_month = if month == 1 { 12 } else { month - 1 };
    let prev_year = if month == 1 { year - 1 } else { year };
    let next_month = if month == 12 { 1 } else { month + 1 };
    let next_year = if month == 12 { year + 1 } else { year };

    preload_holidays(ctx, prev_year, prev_month);
    preload_holidays(ctx, year, month);
    preload_holidays(ctx, next_year, next_month);

    let months = vec![
        MonthData::new(ctx, prev_year, prev_month),
        MonthData::new(ctx, year, month),
        MonthData::new(ctx, next_year, next_month),
    ];

    if ctx.vertical {
        print_three_months_vertical(ctx, &months);
    } else {
        print_months_side_by_side(ctx, &months);
    }
}

/// Print multiple months side by side in horizontal layout.
pub fn print_months_side_by_side(ctx: &CalContext, months: &[MonthData]) {
    let grids: Vec<Vec<String>> = months.iter().map(|m| format_month_grid(ctx, m)).collect();
    let max_height = grids.iter().map(|g| g.len()).max().unwrap_or(0);

    let month_width: usize = if ctx.julian {
        27
    } else if ctx.week_numbers {
        23
    } else {
        20
    };

    for row in 0..max_height {
        let mut line = String::new();
        for (i, grid) in grids.iter().enumerate() {
            if row < grid.len() {
                let text = &grid[row];
                let text_width = text.width();
                line.push_str(text);
                let padding = month_width.saturating_sub(text_width);
                for _ in 0..padding {
                    line.push(' ');
                }
                if i < grids.len() - 1 {
                    for _ in 0..ctx.gutter_width {
                        line.push(' ');
                    }
                }
            } else {
                let width = if i < grids.len() - 1 {
                    month_width + ctx.gutter_width
                } else {
                    month_width
                };
                for _ in 0..width {
                    line.push(' ');
                }
            }
        }
        println!("{}", line);
    }
}

/// Print all 12 months of a year.
pub fn print_year(ctx: &CalContext, year: i32) {
    if ctx.vertical {
        println!("{}", center_text(&year.to_string(), 62));
    } else {
        println!("{}", center_text(&year.to_string(), 66));
    }
    println!();

    if ctx.holidays {
        preload_year_holidays(ctx, year);
    }

    let mut month_ctx = ctx.clone();
    month_ctx.show_year_in_header = false;
    month_ctx.gutter_width = if ctx.vertical { 1 } else { GUTTER_WIDTH_YEAR };

    // Group months into rows of 3
    let mut month_rows = Vec::new();
    for month_row in 0..4 {
        let mut months = Vec::new();
        for col in 0..3 {
            let month = (month_row * 3 + col + 1) as u32;
            if month <= 12 {
                months.push(MonthData::new(&month_ctx, year, month));
            }
        }
        if !months.is_empty() {
            month_rows.push(months);
        }
    }

    if ctx.vertical {
        for months in month_rows.iter() {
            print_three_months_vertical(&month_ctx, months);
        }
    } else {
        for months in month_rows.iter() {
            print_months_side_by_side(&month_ctx, months);
        }
    }
}

/// Print three months in vertical layout.
pub fn print_three_months_vertical(ctx: &CalContext, months: &[MonthData]) {
    let month_width = 18;

    // Print headers
    for (i, month) in months.iter().enumerate() {
        let month_name = get_month_name(month.month);
        let header = if ctx.show_year_in_header {
            format!("{} {}", month_name, month.year)
        } else {
            month_name.to_string()
        };
        let padded_header = if i == 0 {
            format!(
                "    {:<width$}{}",
                header,
                " ".repeat(ctx.gutter_width),
                width = month_width
            )
        } else {
            format!(
                "{:<width$}{}",
                header,
                " ".repeat(ctx.gutter_width),
                width = month_width
            )
        };
        if ctx.color {
            print!("{}{}{}", COLOR_TEAL, padded_header, COLOR_RESET);
        } else {
            print!("{}", padded_header);
        }
    }
    println!();

    let locale = get_system_locale();
    let weekday_order = get_weekday_order(ctx.week_start);
    let weekday_names: Vec<String> = weekday_order
        .iter()
        .map(|&w| get_weekday_short_name(w, locale))
        .collect();

    for (row, &weekday) in weekday_order.iter().enumerate() {
        let day_short = &weekday_names[row];
        if ctx.color {
            print!("{}{}{}", COLOR_SAND_YELLOW, day_short, COLOR_RESET);
        } else {
            print!("{}", day_short);
        }

        for (month_idx, month) in months.iter().enumerate() {
            if month_idx > 0 {
                for _ in 0..ctx.gutter_width {
                    print!(" ");
                }
            }

            for week in 0..6 {
                let day_idx = (weekday as usize) + 7 * week;
                if day_idx < month.days.len() {
                    if let Some(day) = month.days[day_idx] {
                        print_day_vertical(ctx, day, month, weekday);
                    } else {
                        print!("   ");
                    }
                }
            }
        }
        println!();
    }
    println!();
}

/// Print 12 months starting from a given month (--twelve mode).
pub fn print_twelve_months(ctx: &CalContext, start_year: i32, start_month: u32) {
    // Preload holiday data for all 12 months
    if ctx.holidays {
        for i in 0..12 {
            let mut month = start_month + i;
            let mut year = start_year;
            while month > 12 {
                month -= 12;
                year += 1;
            }
            preload_holidays(ctx, year, month);
        }
    }

    let mut month_ctx = ctx.clone();
    month_ctx.show_year_in_header = true;
    month_ctx.gutter_width = GUTTER_WIDTH_YEAR;

    let months = (0..12)
        .map(|i| {
            let mut month = start_month + i;
            let mut year = start_year;
            while month > 12 {
                month -= 12;
                year += 1;
            }
            MonthData::new(&month_ctx, year, month)
        })
        .collect::<Vec<_>>();

    if ctx.vertical {
        for month_data in &months {
            print_month_vertical(&month_ctx, month_data, true);
            println!();
        }
    } else {
        for chunk in months.chunks(3) {
            print_months_side_by_side(&month_ctx, chunk);
        }
    }
}

/// Print a specified number of months (-n mode).
pub fn print_months_count(
    ctx: &CalContext,
    start_year: i32,
    start_month: u32,
    count: u32,
) -> Result<(), String> {
    let months_per_row = ctx.months_per_row();

    // Calculate start month for span mode (center around current month)
    let (actual_start_year, actual_start_month) = if ctx.span && count > 1 {
        let total_months = start_year * 12 + (start_month - 1) as i32;
        let half = (count as i32 - 1) / 2;
        let start = total_months - half;
        let year = start.div_euclid(12);
        let month = (start.rem_euclid(12) + 1) as u32;
        (year, month)
    } else {
        (start_year, start_month)
    };

    // Preload holiday data for all months
    if ctx.holidays {
        for i in 0..count {
            let mut month = actual_start_month + i;
            let mut year = actual_start_year;
            while month > 12 {
                month -= 12;
                year += 1;
            }
            while month < 1 {
                month += 12;
                year -= 1;
            }
            preload_holidays(ctx, year, month);
        }
    }

    let months = (0..count)
        .map(|i| {
            let mut month = actual_start_month + i;
            let mut year = actual_start_year;
            while month > 12 {
                month -= 12;
                year += 1;
            }
            while month < 1 {
                month += 12;
                year -= 1;
            }
            MonthData::new(ctx, year, month)
        })
        .collect::<Vec<_>>();

    if ctx.vertical {
        for month_data in &months {
            print_month_vertical(ctx, month_data, true);
            println!();
        }
    } else {
        for chunk in months.chunks(months_per_row as usize) {
            print_months_side_by_side(ctx, chunk);
        }
    }

    Ok(())
}
