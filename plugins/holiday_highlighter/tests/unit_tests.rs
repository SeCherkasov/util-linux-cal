//! Unit tests for holiday_highlighter plugin.

use std::sync::Mutex;

use holiday_highlighter::get_country_from_locale;

/// Mutex to serialize tests that modify environment variables.
/// `set_var` is not thread-safe, so locale tests must not run in parallel.
/// We use `lock().unwrap_or_else(|e| e.into_inner())` to recover from poison.
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

/// Reset all locale env vars to a clean state, then set `LC_ALL` to the given value.
fn set_locale(lc_all: &str) {
    unsafe {
        std::env::set_var("LC_ALL", lc_all);
        std::env::remove_var("LC_TIME");
        std::env::remove_var("LANG");
    }
}

/// Remove all locale env vars.
fn clear_locale() {
    unsafe {
        std::env::remove_var("LC_ALL");
        std::env::remove_var("LC_TIME");
        std::env::remove_var("LANG");
    }
}

// ---------------------------------------------------------------------------
// Country detection from locale
// ---------------------------------------------------------------------------

#[test]
fn country_from_locale_ru() {
    let _guard = lock_env();
    set_locale("ru_RU.UTF-8");
    assert_eq!(get_country_from_locale(), "RU");
}

#[test]
fn country_from_locale_us() {
    let _guard = lock_env();
    set_locale("en_US.UTF-8");
    assert_eq!(get_country_from_locale(), "US");
}

#[test]
fn country_from_locale_by() {
    let _guard = lock_env();
    set_locale("be_BY.UTF-8");
    assert_eq!(get_country_from_locale(), "BY");
}

#[test]
fn country_from_locale_kz() {
    let _guard = lock_env();
    set_locale("kk_KZ.UTF-8");
    assert_eq!(get_country_from_locale(), "KZ");
}

#[test]
fn country_from_locale_fallback_to_us() {
    let _guard = lock_env();
    clear_locale();
    // When no locale vars are set, the function defaults to "en_US.UTF-8" -> "US"
    assert_eq!(get_country_from_locale(), "US");
}

#[test]
fn country_from_locale_lc_time_fallback() {
    let _guard = lock_env();
    clear_locale();
    unsafe { std::env::set_var("LC_TIME", "tr_TR.UTF-8") };
    assert_eq!(get_country_from_locale(), "TR");
}
