//! Plugin API for dynamic loading of holiday highlighter.

use libc::{c_char, c_int};
use std::ffi::{CStr, CString};
use std::path::Path;

/// Handle to a loaded plugin.
pub struct PluginHandle {
    #[allow(dead_code)]
    lib: libloading::Library,
    get_holidays_fn: libloading::Symbol<
        'static,
        unsafe extern "C" fn(c_int, c_int, *const c_char) -> *mut c_char,
    >,
    free_holidays_fn: libloading::Symbol<'static, unsafe extern "C" fn(*mut c_char)>,
    get_country_fn: libloading::Symbol<'static, unsafe extern "C" fn() -> *mut c_char>,
    free_country_fn: libloading::Symbol<'static, unsafe extern "C" fn(*mut c_char)>,
    is_holiday_fn: libloading::Symbol<
        'static,
        unsafe extern "C" fn(c_int, c_int, c_int, *const c_char) -> c_int,
    >,
    get_year_holidays_fn:
        libloading::Symbol<'static, unsafe extern "C" fn(c_int, *const c_char) -> *mut c_char>,
}

impl PluginHandle {
    /// Load plugin from path.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, libloading::Error> {
        let lib = unsafe { libloading::Library::new(path.as_ref())? };

        unsafe {
            let get_holidays_fn: libloading::Symbol<
                unsafe extern "C" fn(c_int, c_int, *const c_char) -> *mut c_char,
            > = lib.get(b"plugin_get_holidays")?;

            let free_holidays_fn: libloading::Symbol<unsafe extern "C" fn(*mut c_char)> =
                lib.get(b"plugin_free_holidays")?;

            let get_country_fn: libloading::Symbol<unsafe extern "C" fn() -> *mut c_char> =
                lib.get(b"plugin_get_country_from_locale")?;

            let free_country_fn: libloading::Symbol<unsafe extern "C" fn(*mut c_char)> =
                lib.get(b"plugin_free_country")?;

            let is_holiday_fn: libloading::Symbol<
                unsafe extern "C" fn(c_int, c_int, c_int, *const c_char) -> c_int,
            > = lib.get(b"plugin_is_holiday")?;

            let get_year_holidays_fn: libloading::Symbol<
                unsafe extern "C" fn(c_int, *const c_char) -> *mut c_char,
            > = lib.get(b"plugin_get_year_holidays")?;

            // Extend lifetime to match struct
            let get_holidays_fn: libloading::Symbol<
                'static,
                unsafe extern "C" fn(c_int, c_int, *const c_char) -> *mut c_char,
            > = std::mem::transmute(get_holidays_fn);
            let free_holidays_fn: libloading::Symbol<'static, unsafe extern "C" fn(*mut c_char)> =
                std::mem::transmute(free_holidays_fn);
            let get_country_fn: libloading::Symbol<'static, unsafe extern "C" fn() -> *mut c_char> =
                std::mem::transmute(get_country_fn);
            let free_country_fn: libloading::Symbol<'static, unsafe extern "C" fn(*mut c_char)> =
                std::mem::transmute(free_country_fn);
            let is_holiday_fn: libloading::Symbol<
                'static,
                unsafe extern "C" fn(c_int, c_int, c_int, *const c_char) -> c_int,
            > = std::mem::transmute(is_holiday_fn);
            let get_year_holidays_fn: libloading::Symbol<
                'static,
                unsafe extern "C" fn(c_int, *const c_char) -> *mut c_char,
            > = std::mem::transmute(get_year_holidays_fn);

            Ok(PluginHandle {
                lib,
                get_holidays_fn,
                free_holidays_fn,
                get_country_fn,
                free_country_fn,
                is_holiday_fn,
                get_year_holidays_fn,
            })
        }
    }

    /// Get holiday data for a specific month.
    ///
    /// Returns string where each character represents a day:
    /// - '0' = working day, '1' = weekend, '2' = shortened, '8' = public holiday
    pub fn get_holidays(&self, year: i32, month: u32, country: &str) -> Option<String> {
        let country_cstr = CString::new(country).ok()?;

        unsafe {
            let result =
                (self.get_holidays_fn)(year as c_int, month as c_int, country_cstr.as_ptr());
            if result.is_null() {
                return None;
            }

            let rust_str = CStr::from_ptr(result).to_str().ok()?.to_string();
            (self.free_holidays_fn)(result);
            Some(rust_str)
        }
    }

    /// Get holiday data for entire year.
    pub fn get_year_holidays(&self, year: i32, country: &str) -> Option<String> {
        let country_cstr = CString::new(country).ok()?;

        unsafe {
            let result = (self.get_year_holidays_fn)(year as c_int, country_cstr.as_ptr());
            if result.is_null() {
                return None;
            }

            let rust_str = CStr::from_ptr(result).to_str().ok()?.to_string();
            (self.free_holidays_fn)(result);
            Some(rust_str)
        }
    }

    /// Get country code from system locale.
    pub fn get_country_from_locale(&self) -> String {
        unsafe {
            let result = (self.get_country_fn)();
            if result.is_null() {
                return "RU".to_string();
            }

            let country = CStr::from_ptr(result).to_str().unwrap_or("RU").to_string();
            (self.free_country_fn)(result);
            country
        }
    }

    /// Check if a specific day is a holiday.
    ///
    /// Returns: 0=working, 1=weekend, 2=shortened, 8=public, -1=error
    pub fn is_holiday(&self, year: i32, month: u32, day: u32, country: &str) -> i32 {
        let country_cstr = CString::new(country).unwrap();

        unsafe {
            (self.is_holiday_fn)(
                year as c_int,
                month as c_int,
                day as c_int,
                country_cstr.as_ptr(),
            )
        }
    }
}

/// Try to load the holiday plugin from standard locations.
pub fn try_load_plugin() -> Option<PluginHandle> {
    let search_paths = [
        // Build directory (development)
        "./target/debug/libholiday_highlighter.so",
        "./target/release/libholiday_highlighter.so",
        // User local directory
        "~/.local/lib/cal/plugins/libholiday_highlighter.so",
        // System directory
        "/usr/lib/cal/plugins/libholiday_highlighter.so",
        "/usr/local/lib/cal/plugins/libholiday_highlighter.so",
        // Relative to executable
        "./plugins/libholiday_highlighter.so",
        "./libholiday_highlighter.so",
    ];

    for path in &search_paths {
        let expanded = shellexpand::tilde(path);
        if let Ok(handle) = PluginHandle::load(expanded.as_ref()) {
            return Some(handle);
        }
    }

    None
}
