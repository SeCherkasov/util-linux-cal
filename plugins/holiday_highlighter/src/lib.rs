//! Holiday Highlighter Plugin for cal.
//!
//! Fetches holiday data from isdayoff.ru API for multiple countries.

use libc::{c_char, c_int};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::sync::{LazyLock, Mutex};

pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");
pub const PLUGIN_VERSION: &str = env!("CARGO_PKG_VERSION");

const API_URL_YEAR: &str = "https://isdayoff.ru/api/getdata";
const API_URL_MONTH: &str = "https://isdayoff.ru/api/getdata";

/// Supported countries with their locale mappings.
const SUPPORTED_COUNTRIES: &[(&str, &[&str])] = &[
    ("RU", &["ru_RU", "ru_BY", "ru_KZ", "ru_UZ", "ru_LV"]),
    ("BY", &["be_BY", "ru_BY"]),
    ("KZ", &["kk_KZ", "ru_KZ"]),
    ("US", &["en_US", "en"]),
    ("UZ", &["uz_UZ", "ru_UZ"]),
    ("TR", &["tr_TR"]),
    ("LV", &["lv_LV", "ru_LV"]),
];

type CacheKey = (i32, u32, String);
static CACHE: LazyLock<Mutex<HashMap<CacheKey, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Initialize the plugin (optional, cache is lazily initialized).
#[unsafe(no_mangle)]
pub extern "C" fn plugin_init() {}

/// Get plugin name (do not free returned pointer).
#[unsafe(no_mangle)]
pub extern "C" fn plugin_get_name() -> *const c_char {
    static NAME: LazyLock<CString> = LazyLock::new(|| CString::new(PLUGIN_NAME).unwrap());
    NAME.as_ptr()
}

/// Get plugin version (do not free returned pointer).
#[unsafe(no_mangle)]
pub extern "C" fn plugin_get_version() -> *const c_char {
    static VERSION: LazyLock<CString> = LazyLock::new(|| CString::new(PLUGIN_VERSION).unwrap());
    VERSION.as_ptr()
}

/// Get holiday data for a specific month.
///
/// Returns string where each character represents a day:
/// - '0' = working day, '1' = weekend, '2' = shortened, '8' = public holiday
///
/// # Safety
/// `country_code` must be a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn plugin_get_holidays(
    year: c_int,
    month: c_int,
    country_code: *const c_char,
) -> *mut c_char {
    let country = unsafe { parse_country(country_code) };
    let key = (year, month as u32, country.clone());

    if let Some(data) = CACHE.lock().unwrap().get(&key) {
        return CString::new(data.as_str()).unwrap().into_raw();
    }

    let data = fetch_holidays(year, month as u32, &country).unwrap_or_default();
    CACHE.lock().unwrap().insert(key, data.clone());

    CString::new(data).unwrap().into_raw()
}

unsafe fn parse_country(country_code: *const c_char) -> String {
    unsafe {
        CStr::from_ptr(country_code)
            .to_str()
            .unwrap_or("RU")
            .to_string()
    }
}

/// Free memory allocated by plugin_get_holidays.
///
/// # Safety
/// `ptr` must be returned by `plugin_get_holidays`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn plugin_free_holidays(ptr: *mut c_char) {
    if !ptr.is_null() {
        let _ = unsafe { CString::from_raw(ptr) };
    }
}

/// Get country code from system locale.
#[unsafe(no_mangle)]
pub extern "C" fn plugin_get_country_from_locale() -> *mut c_char {
    let country = get_country_from_locale();
    CString::new(country).unwrap().into_raw()
}

/// Free memory allocated by plugin_get_country_from_locale.
///
/// # Safety
/// `ptr` must be returned by `plugin_get_country_from_locale`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn plugin_free_country(ptr: *mut c_char) {
    if !ptr.is_null() {
        let _ = unsafe { CString::from_raw(ptr) };
    }
}

/// Check if a specific day is a holiday.
///
/// Returns: 0=working, 1=weekend, 2=shortened, 8=public, -1=error
///
/// # Safety
/// `country_code` must be a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn plugin_is_holiday(
    year: c_int,
    month: c_int,
    day: c_int,
    country_code: *const c_char,
) -> c_int {
    let country = unsafe { parse_country(country_code) };
    let holidays = fetch_holidays(year, month as u32, &country);

    match holidays {
        Some(data) => {
            let day_idx = (day - 1) as usize;
            if day_idx < data.len() {
                data.chars()
                    .nth(day_idx)
                    .and_then(|c| c.to_digit(10).map(|d| d as c_int))
                    .unwrap_or(-1)
            } else {
                -1
            }
        }
        None => -1,
    }
}

/// Get holiday data for entire year.
///
/// # Safety
/// `country_code` must be a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn plugin_get_year_holidays(
    year: c_int,
    country_code: *const c_char,
) -> *mut c_char {
    let country = unsafe { parse_country(country_code) };
    let data = fetch_holidays_year(year, &country).unwrap_or_default();
    CString::new(data).unwrap().into_raw()
}

/// Fetch holiday data for a month from isdayoff.ru API.
fn fetch_holidays(year: i32, month: u32, country: &str) -> Option<String> {
    let url = format!(
        "{}?year={}&month={:02}&cc={}&pre=1",
        API_URL_MONTH,
        year,
        month,
        country.to_lowercase()
    );

    match ureq::get(&url).call() {
        Ok(response) => response.into_body().read_to_string().ok(),
        Err(_) => None,
    }
}

/// Fetch holiday data for entire year from isdayoff.ru API.
pub fn fetch_holidays_year(year: i32, country: &str) -> Option<String> {
    let url = format!(
        "{}?year={}&cc={}&pre=1",
        API_URL_YEAR,
        year,
        country.to_lowercase()
    );

    match ureq::get(&url).call() {
        Ok(response) => response.into_body().read_to_string().ok(),
        Err(_) => None,
    }
}

/// Determine country code from system locale.
pub fn get_country_from_locale() -> String {
    let locale = std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LC_TIME"))
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_else(|_| "en_US.UTF-8".to_string());

    let locale_name = locale
        .split('.')
        .next()
        .unwrap_or(&locale)
        .split('@')
        .next()
        .unwrap_or(&locale);

    // Match against supported countries
    for (country, locales) in SUPPORTED_COUNTRIES {
        for &supported_locale in *locales {
            if locale_name == supported_locale {
                return country.to_string();
            }
        }
    }

    // Extract country code from locale (e.g., "en_US" -> "US")
    if let Some(underscore_pos) = locale_name.find('_') {
        let country_code = &locale_name[underscore_pos + 1..];
        for (country, _) in SUPPORTED_COUNTRIES {
            if *country == country_code {
                return country_code.to_string();
            }
        }
    }

    "RU".to_string()
}
