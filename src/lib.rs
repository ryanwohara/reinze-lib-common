pub mod author;
pub mod database;
pub mod source;

use format_num::NumberFormat;
#[allow(unused_imports)]
use mysql::{prelude::*, *};
use regex::Regex;

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

#[derive(Clone)]
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

    pub fn c0<T>(&self, s: T, i: u32) -> String
    where
        T: ToString,
    {
        let color = match i {
            1 => &self.c1,
            2 | _ => &self.c2,
        };

        format!("\x03{}{}", color.to_string(), s.to_string())
    }

    pub fn c1<T>(&self, str: T) -> String
    where
        T: ToString,
    {
        self.c0(str, 1)
    }

    pub fn c2<T>(&self, str: T) -> String
    where
        T: ToString,
    {
        self.c0(str, 2)
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

// Gray
// c1
pub fn c1<T>(s: T) -> String
where
    T: ToString,
{
    format!("\x0314{}", s.to_string())
}

// Red
// c2
pub fn c2<T>(s: T) -> String
where
    T: ToString,
{
    format!("\x0304{}", s.to_string())
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

// A function for wrapping a string in brackets that are colored gray
// l
pub fn l<T>(s: T) -> String
where
    T: ToString,
{
    format!("{}{}{}", c1("["), c2(s), c1("]"))
}

// A function for wrapping a string in parentheses that are colored gray
// p
pub fn p<T>(s: T) -> String
where
    T: ToString,
{
    format!("{}{}{}", c1("("), c2(s), c1(")"))
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

// Removes the trailing zeroes from a string (intended to be used on a float->&str that may have commas)
pub fn remove_trailing_zeroes(str: &str) -> String {
    let re = Regex::new(r"\.?0+$").unwrap();

    re.replace_all(str, "").to_string()
}

pub fn unranked(v: Vec<String>) -> String {
    if v.is_empty() {
        c2("Unranked")
    } else {
        v.join(&c1(" | "))
    }
}

pub fn not_found(v: Vec<String>) -> String {
    if v.is_empty() {
        c2("Not found")
    } else {
        v.join(&c1(" | "))
    }
}

pub fn convert_split_to_string(split: Vec<&str>) -> Vec<String> {
    split.into_iter().map(|s| s.to_string()).collect()
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
    fn test_c1() {
        assert_eq!(c1("hello"), "\x0314hello");
        assert_eq!(c1("Hello"), "\x0314Hello");
        assert_eq!(c1("HELLO"), "\x0314HELLO");
        assert_eq!(c1("hELLO"), "\x0314hELLO");
        assert_eq!(c1("hElLo"), "\x0314hElLo");
        assert_eq!(c1("123"), "\x0314123");
        assert_eq!(c1(""), "\x0314");
    }

    #[test]
    fn test_c2() {
        assert_eq!(c2("hello"), "\x0304hello");
        assert_eq!(c2("Hello"), "\x0304Hello");
        assert_eq!(c2("HELLO"), "\x0304HELLO");
        assert_eq!(c2("hELLO"), "\x0304hELLO");
        assert_eq!(c2("hElLo"), "\x0304hElLo");
        assert_eq!(c2("123"), "\x0304123");
        assert_eq!(c2(""), "\x0304");
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
    fn test_l() {
        assert_eq!(l("hello"), "\x0314[\x0304hello\x0314]");
        assert_eq!(l("Hello"), "\x0314[\x0304Hello\x0314]");
        assert_eq!(l("HELLO"), "\x0314[\x0304HELLO\x0314]");
        assert_eq!(l("hELLO"), "\x0314[\x0304hELLO\x0314]");
        assert_eq!(l("hElLo"), "\x0314[\x0304hElLo\x0314]");
        assert_eq!(l("123"), "\x0314[\x0304123\x0314]");
        assert_eq!(l(""), "\x0314[\x0304\x0314]");
    }

    #[test]
    fn test_p() {
        assert_eq!(p("hello"), "\x0314(\x0304hello\x0314)");
        assert_eq!(p("Hello"), "\x0314(\x0304Hello\x0314)");
        assert_eq!(p("HELLO"), "\x0314(\x0304HELLO\x0314)");
        assert_eq!(p("hELLO"), "\x0314(\x0304hELLO\x0314)");
        assert_eq!(p("hElLo"), "\x0314(\x0304hElLo\x0314)");
        assert_eq!(p("123"), "\x0314(\x0304123\x0314)");
        assert_eq!(p(""), "\x0314(\x0304\x0314)");
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
}
