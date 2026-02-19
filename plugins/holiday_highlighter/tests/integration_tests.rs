//! Integration tests for holiday_highlighter plugin.

use holiday_highlighter::{
    PLUGIN_NAME, PLUGIN_VERSION, get_country_from_locale, plugin_get_name, plugin_get_version,
};
use std::ffi::CStr;

#[test]
fn test_get_country_from_locale_ru() {
    unsafe {
        std::env::set_var("LC_ALL", "ru_RU.UTF-8");
    }
    assert_eq!(get_country_from_locale(), "RU");
}

#[test]
fn test_get_country_from_locale_us() {
    unsafe {
        std::env::set_var("LC_ALL", "en_US.UTF-8");
    }
    assert_eq!(get_country_from_locale(), "US");
}

#[test]
fn test_get_country_from_locale_by() {
    unsafe {
        std::env::set_var("LC_ALL", "be_BY.UTF-8");
    }
    assert_eq!(get_country_from_locale(), "BY");
}

#[test]
fn test_get_country_from_locale_fallback() {
    unsafe {
        std::env::set_var("LC_ALL", "");
        std::env::set_var("LC_TIME", "");
        std::env::set_var("LANG", "");
    }
    assert_eq!(get_country_from_locale(), "RU");
}

#[test]
fn test_plugin_metadata_from_cargo() {
    assert_eq!(PLUGIN_NAME, "holiday_highlighter");
    assert_eq!(PLUGIN_VERSION, "0.1.0");
}

#[test]
fn test_plugin_get_name_version() {
    unsafe {
        let name = CStr::from_ptr(plugin_get_name()).to_str().unwrap();
        let version = CStr::from_ptr(plugin_get_version()).to_str().unwrap();
        assert_eq!(name, "holiday_highlighter");
        assert_eq!(version, "0.1.0");
    }
}
