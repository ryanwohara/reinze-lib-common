pub mod author;
pub mod database;
pub mod snapshot;
pub mod source;

use format_num::NumberFormat;
#[allow(unused_imports)]
use mysql::{prelude::*, *};
use regex::Regex;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::LazyLock;

#[repr(C)]
pub struct PluginContext {
    pub cmd: *const c_char,
    pub param: *const c_char,
    pub author: *const c_char,
    pub color: extern "C" fn(*const c_char, *const c_char) -> ColorResult,
    // Appended last for ABI compatibility: plugins built against the older
    // 4-field layout read the same offsets and simply ignore this field.
    pub channel: *const c_char,
}

#[repr(C)]
pub struct ColorResult {
    pub c1: *const c_char,
    pub c2: *const c_char,
}

impl From<&Colors> for ColorResult {
    fn from(colors: &Colors) -> Self {
        let c1 = CString::new(colors.c1.to_string()).unwrap().into_raw();
        let c2 = CString::new(colors.c2.to_string()).unwrap().into_raw();

        ColorResult { c1, c2 }
    }
}

impl Default for ColorResult {
    fn default() -> Self {
        Self::from(&Colors::default())
    }
}

pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f
            .to_uppercase()
            .chain(c.flat_map(|c| c.to_lowercase()))
            .collect(),
    }
}

#[derive(Clone, Debug)]
pub struct Colors {
    pub c1: String,
    pub c2: String,
}

impl Colors {
    pub fn color1() -> String {
        "14".to_string()
    }

    pub fn color2() -> String {
        "04".to_string()
    }

    pub fn init() {
        author::cache::init();
    }
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            c1: Self::color1(),
            c2: Self::color2(),
        }
    }
}


// Red
// c3
pub fn c3<T>(s: T) -> String
where
    T: ToString,
{
    format!("\x0305{}", s.to_string())
}

// Green
// c4
pub fn c4<T>(s: T) -> String
where
    T: ToString,
{
    format!("\x0303{}", s.to_string())
}

// Yellow
// c5
pub fn c5<T>(s: T) -> String
where
    T: ToString,
{
    format!("\x0307{}", s.to_string())
}

// Adds commas to a number
pub fn commas(n: f64, f: &str) -> String {
    let num = NumberFormat::new();

    num.format(&format!(",{}", f), n)
}

// Adds commas to a string
pub fn commas_from_string(n: &str, f: &str) -> String {
    let n = n.parse::<f64>().unwrap_or(0.0);

    commas(n, f)
}

static TRAILING_ZEROES_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\.?0+$").unwrap());

// Removes the trailing zeroes from a string (intended to be used on a float->&str that may have commas)
pub fn remove_trailing_zeroes(str: &str) -> String {
    TRAILING_ZEROES_RE.replace_all(str, "").to_string()
}


pub fn convert_split_to_string(split: Vec<&str>) -> Vec<String> {
    split.into_iter().map(|s| s.to_string()).collect()
}

#[allow(dead_code)]
pub fn to_str_or_default(ptr: *const c_char) -> String {
    let cstr = unsafe { CStr::from_ptr(ptr) };
    cstr.to_str().unwrap_or_default().to_owned()
}

#[cfg(test)]
mod tests {
    // import names from outer (for mod tests) scope
    use super::*;

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("hello"), "Hello");
        assert_eq!(capitalize("Hello"), "Hello");
        assert_eq!(capitalize("HELLO"), "Hello");
        assert_eq!(capitalize("hELLO"), "Hello");
        assert_eq!(capitalize("hElLo"), "Hello");
        assert_eq!(capitalize("123"), "123");
        assert_eq!(capitalize(""), "");
    }


    #[test]
    fn test_c3() {
        assert_eq!(c3("hello"), "\x0305hello");
        assert_eq!(c3("Hello"), "\x0305Hello");
        assert_eq!(c3("HELLO"), "\x0305HELLO");
        assert_eq!(c3("hELLO"), "\x0305hELLO");
        assert_eq!(c3("hElLo"), "\x0305hElLo");
        assert_eq!(c3("123"), "\x0305123");
        assert_eq!(c3(""), "\x0305");
    }

    #[test]
    fn test_c4() {
        assert_eq!(c4("hello"), "\x0303hello");
        assert_eq!(c4("Hello"), "\x0303Hello");
        assert_eq!(c4("HELLO"), "\x0303HELLO");
        assert_eq!(c4("hELLO"), "\x0303hELLO");
        assert_eq!(c4("hElLo"), "\x0303hElLo");
        assert_eq!(c4("123"), "\x0303123");
        assert_eq!(c4(""), "\x0303");
    }

    #[test]
    fn test_c5() {
        assert_eq!(c5("hello"), "\x0307hello");
        assert_eq!(c5("Hello"), "\x0307Hello");
        assert_eq!(c5("HELLO"), "\x0307HELLO");
        assert_eq!(c5("hELLO"), "\x0307hELLO");
        assert_eq!(c5("hElLo"), "\x0307hElLo");
        assert_eq!(c5("123"), "\x0307123");
        assert_eq!(c5(""), "\x0307");
    }


    #[test]
    fn test_commas() {
        assert_eq!(commas(0.0, "d"), "0");
        assert_eq!(commas(1.0, "d"), "1");
        assert_eq!(commas(10.0, "d"), "10");
        assert_eq!(commas(100.0, "d"), "100");
        assert_eq!(commas(1000.0, "d"), "1,000");
        assert_eq!(commas(10000.0, "d"), "10,000");
        assert_eq!(commas(100000.0, "d"), "100,000");
        assert_eq!(commas(1000000.0, "d"), "1,000,000");
        assert_eq!(commas(10000000.0, "d"), "10,000,000");
        assert_eq!(commas(100000000.0, "d"), "100,000,000");
        assert_eq!(commas(1000000000.0, "d"), "1,000,000,000");
        assert_eq!(commas(10000000000.0, "d"), "10,000,000,000");
        assert_eq!(commas(100000000000.0, "d"), "100,000,000,000");
        assert_eq!(commas(1000000000000.0, "d"), "1,000,000,000,000");
        assert_eq!(commas(10000000000000.0, "d"), "10,000,000,000,000");
        assert_eq!(commas(100000000000000.0, "d"), "100,000,000,000,000");
        assert_eq!(commas(1000000000000000.0, "d"), "1,000,000,000,000,000");
        assert_eq!(commas(10000000000000000.0, "d"), "10,000,000,000,000,000");
        assert_eq!(commas(100000000000000000.0, "d"), "100,000,000,000,000,000");
        assert_eq!(
            commas(1000000000000000000.0, "d"),
            "1,000,000,000,000,000,000"
        );
        assert_eq!(
            commas(10000000000000000000.0, "d"),
            "10,000,000,000,000,000,000"
        );
        assert_eq!(
            commas(100000000000000000000.0, "d"),
            "100,000,000,000,000,000,000"
        );
    }

    #[test]
    fn test_commas_from_string() {
        assert_eq!(commas_from_string("0", "d"), "0");
        assert_eq!(commas_from_string("1", "d"), "1");
        assert_eq!(commas_from_string("10", "d"), "10");
        assert_eq!(commas_from_string("100", "d"), "100");
        assert_eq!(commas_from_string("1000", "d"), "1,000");
        assert_eq!(commas_from_string("10000", "d"), "10,000");
        assert_eq!(commas_from_string("100000", "d"), "100,000");
        assert_eq!(commas_from_string("1000000", "d"), "1,000,000");
        assert_eq!(commas_from_string("10000000", "d"), "10,000,000");
        assert_eq!(commas_from_string("100000000", "d"), "100,000,000");
        assert_eq!(commas_from_string("1000000000", "d"), "1,000,000,000");
        assert_eq!(commas_from_string("10000000000", "d"), "10,000,000,000");
        assert_eq!(commas_from_string("100000000000", "d"), "100,000,000,000");
        assert_eq!(
            commas_from_string("1000000000000", "d"),
            "1,000,000,000,000"
        );
        assert_eq!(
            commas_from_string("10000000000000", "d"),
            "10,000,000,000,000"
        );
        assert_eq!(
            commas_from_string("100000000000000", "d"),
            "100,000,000,000,000"
        );
        assert_eq!(
            commas_from_string("1000000000000000", "d"),
            "1,000,000,000,000,000"
        );
        assert_eq!(
            commas_from_string("10000000000000000", "d"),
            "10,000,000,000,000,000"
        );
        assert_eq!(
            commas_from_string("100000000000000000", "d"),
            "100,000,000,000,000,000"
        );
        assert_eq!(
            commas_from_string("1000000000000000000", "d"),
            "1,000,000,000,000,000,000"
        );
        assert_eq!(
            commas_from_string("10000000000000000000", "d"),
            "10,000,000,000,000,000,000"
        );
        assert_eq!(
            commas_from_string("100000000000000000000", "d"),
            "100,000,000,000,000,000,000"
        );
    }

    #[test]
    fn test_remove_trailing_zeroes() {
        assert_eq!(remove_trailing_zeroes("0.00000"), "0");
        assert_eq!(remove_trailing_zeroes("1.00000"), "1");
        assert_eq!(remove_trailing_zeroes("10.00000"), "10");
        assert_eq!(remove_trailing_zeroes("100.00000"), "100");
        assert_eq!(remove_trailing_zeroes("1,000.00000"), "1,000");
        assert_eq!(remove_trailing_zeroes("10,000.00000"), "10,000");
        assert_eq!(remove_trailing_zeroes("100,000.00000"), "100,000");
        assert_eq!(remove_trailing_zeroes("1,000,000.00000"), "1,000,000");
        assert_eq!(remove_trailing_zeroes("10,000,000.00000"), "10,000,000");
        assert_eq!(remove_trailing_zeroes("0.0"), "0");
        assert_eq!(remove_trailing_zeroes("1.0"), "1");
        assert_eq!(remove_trailing_zeroes("10.0"), "10");
        assert_eq!(remove_trailing_zeroes("100.0"), "100");
        assert_eq!(remove_trailing_zeroes("1,000.0"), "1,000");
        assert_eq!(remove_trailing_zeroes("10,000.0"), "10,000");
        assert_eq!(remove_trailing_zeroes("100,000.0"), "100,000");
        assert_eq!(remove_trailing_zeroes("1,000,000.0"), "1,000,000");
        assert_eq!(remove_trailing_zeroes("10,000,000.0"), "10,000,000");
    }

    #[test]
    fn test_remove_trailing_zeroes_partial() {
        assert_eq!(remove_trailing_zeroes("1.50"), "1.5");
        assert_eq!(remove_trailing_zeroes("3.14"), "3.14");
        assert_eq!(remove_trailing_zeroes("2.100"), "2.1");
        assert_eq!(remove_trailing_zeroes("99.990"), "99.99");
    }

    #[test]
    fn test_remove_trailing_zeroes_no_decimal() {
        assert_eq!(remove_trailing_zeroes("100"), "1");
        assert_eq!(remove_trailing_zeroes("42"), "42");
        assert_eq!(remove_trailing_zeroes("7"), "7");
    }

    #[test]
    fn test_remove_trailing_zeroes_empty() {
        assert_eq!(remove_trailing_zeroes(""), "");
    }


    #[test]
    fn test_convert_split_to_string() {
        assert_eq!(
            convert_split_to_string(vec!["a", "b", "c"]),
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }

    #[test]
    fn test_convert_split_to_string_empty() {
        let empty: Vec<&str> = vec![];
        let expected: Vec<String> = vec![];
        assert_eq!(convert_split_to_string(empty), expected);
    }

    #[test]
    fn test_colors_default() {
        let colors = Colors::default();
        assert_eq!(colors.c1, "14");
        assert_eq!(colors.c2, "04");
    }

    #[test]
    fn test_colors_static_methods() {
        assert_eq!(Colors::color1(), "14");
        assert_eq!(Colors::color2(), "04");
    }

    #[test]
    fn test_commas_negative() {
        assert_eq!(commas(-1000.0, "d"), "-1,000");
        assert_eq!(commas(-1.0, "d"), "-1");
    }

    #[test]
    fn test_commas_float_format() {
        assert_eq!(commas(1234.56, ".2f"), "1,234.56");
        assert_eq!(commas(0.5, ".1f"), "0.5");
    }

    #[test]
    fn test_commas_from_string_invalid() {
        assert_eq!(commas_from_string("abc", "d"), "0");
        assert_eq!(commas_from_string("", "d"), "0");
    }

    #[test]
    fn test_commas_from_string_negative() {
        assert_eq!(commas_from_string("-1000", "d"), "-1,000");
    }

    #[test]
    fn test_capitalize_unicode() {
        assert_eq!(capitalize("über"), "Über");
    }

    #[test]
    fn test_capitalize_single_char() {
        assert_eq!(capitalize("a"), "A");
        assert_eq!(capitalize("Z"), "Z");
    }

    #[test]
    fn test_c_functions_accept_non_str() {
        assert_eq!(c3(true), "\x0305true");
        assert_eq!(c4(-1), "\x0303-1");
        assert_eq!(c5(0), "\x03070");
    }
}
