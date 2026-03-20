pub mod cache;

use crate::{ColorResult, Colors};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

pub struct Author {
    pub nick: String,
    pub host: String,
    #[allow(dead_code)]
    pub ident: String,
    #[allow(dead_code)]
    pub address: String,
    pub full: String,
    pub color: extern "C" fn(*const c_char, *const c_char) -> ColorResult,
}

impl Author {
    pub fn create<T>(a: T, f: extern "C" fn(*const c_char, *const c_char) -> ColorResult) -> Self
    where
        T: ToString,
    {
        let author = a.to_string();
        let (nick, mut host) = author.split_once("!").unwrap_or(("", &author));
        let replaced;
        if host.starts_with("~") {
            replaced = host.replace("~", "");
            host = &replaced;
        }

        let (ident, address) = host.split_once("@").unwrap_or(("", &host));

        Self {
            nick: nick.to_string(),
            host: host.to_string(),
            ident: ident.to_string(),
            address: address.to_string(),
            full: author.to_string(),
            color: f,
        }
    }

    pub fn c1<T>(&self, s: T) -> String
    where
        T: ToString,
    {
        let color = unsafe { self.colors() }.c1;

        wrap(s.to_string().as_str(), &color)
    }

    pub fn c2<T>(&self, s: T) -> String
    where
        T: ToString,
    {
        let color = unsafe { self.colors() }.c2;

        wrap(s.to_string().as_str(), &color)
    }

    pub fn l<T>(&self, s: T) -> String
    where
        T: ToString,
    {
        let colors = unsafe { self.colors() };
        format!("{}{}{}", wrap("[", &colors.c1), wrap(&s.to_string(), &colors.c2), wrap("]", &colors.c1))
    }

    pub fn p<T>(&self, s: T) -> String
    where
        T: ToString,
    {
        let colors = unsafe { self.colors() };
        format!("{}{}{}", wrap("(", &colors.c1), wrap(&s.to_string(), &colors.c2), wrap(")", &colors.c1))
    }

    pub unsafe fn colors(&self) -> Colors {
        let host = CString::new(self.host.as_str()).unwrap().into_raw();
        let empty = CString::new("").unwrap().into_raw();

        let results = (self.color)(host, empty);

        let c1 = CStr::from_ptr(results.c1).to_string_lossy().into_owned();
        let c2 = CStr::from_ptr(results.c2).to_string_lossy().into_owned();

        _ = CString::from_raw(host);
        _ = CString::from_raw(empty);
        _ = CString::from_raw(results.c1 as *mut c_char);
        _ = CString::from_raw(results.c2 as *mut c_char);

        Colors { c1, c2 }
    }

    pub fn set_colors(&self, colors: Colors) {
        cache::set(self.host.clone(), colors)
    }

    pub fn clear_colors(&self) {
        self.set_colors(Colors::default())
    }
}

fn wrap(s: &str, color: &str) -> String {
    format!("\x03{}{}", color, s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ColorResult;

    // Stub FFI function for tests — returns default colors
    extern "C" fn stub_color(_host: *const std::os::raw::c_char, _colors: *const std::os::raw::c_char) -> ColorResult {
        ColorResult::default()
    }

    #[test]
    fn test_create_full_prefix() {
        // Tilde is stripped from host before ident@address split
        let author = Author::create("nick!~ident@host.example.com", stub_color);
        assert_eq!(author.nick, "nick");
        assert_eq!(author.ident, "ident");
        assert_eq!(author.address, "host.example.com");
        assert_eq!(author.host, "ident@host.example.com");
    }

    #[test]
    fn test_create_no_tilde() {
        let author = Author::create("nick!ident@host.example.com", stub_color);
        assert_eq!(author.nick, "nick");
        assert_eq!(author.ident, "ident");
        assert_eq!(author.address, "host.example.com");
        assert_eq!(author.host, "ident@host.example.com");
    }

    #[test]
    fn test_create_no_bang() {
        let author = Author::create("justanick", stub_color);
        assert_eq!(author.nick, "");
        assert_eq!(author.host, "justanick");
    }

    #[test]
    fn test_create_empty() {
        let author = Author::create("", stub_color);
        assert_eq!(author.nick, "");
        assert_eq!(author.host, "");
    }

    #[test]
    fn test_create_nick_only_with_bang() {
        let author = Author::create("nick!", stub_color);
        assert_eq!(author.nick, "nick");
        assert_eq!(author.host, "");
    }

    #[test]
    fn test_create_no_at_sign() {
        let author = Author::create("nick!hostonly", stub_color);
        assert_eq!(author.nick, "nick");
        assert_eq!(author.host, "hostonly");
        assert_eq!(author.ident, "");
        assert_eq!(author.address, "hostonly");
    }

    #[test]
    fn test_create_tilde_stripped() {
        let author = Author::create("user!~ident@1.2.3.4", stub_color);
        // Tilde is stripped from the host portion
        assert!(!author.host.contains('~'));
    }

    #[test]
    fn test_create_full_preserved() {
        let input = "nick!~ident@host.com";
        let author = Author::create(input, stub_color);
        assert_eq!(author.full, input);
    }

    #[test]
    fn test_wrap() {
        assert_eq!(wrap("hello", "14"), "\x0314hello");
        assert_eq!(wrap("", "04"), "\x0304");
        assert_eq!(wrap("test", ""), "\x03test");
    }

    #[test]
    fn test_c1_uses_default_color() {
        let author = Author::create("nick!ident@host", stub_color);
        // Default c1 color is "14" (gray)
        assert_eq!(author.c1("text"), "\x0314text");
    }

    #[test]
    fn test_c2_uses_default_color() {
        let author = Author::create("nick!ident@host", stub_color);
        // Default c2 color is "04" (red)
        assert_eq!(author.c2("text"), "\x0304text");
    }

    #[test]
    fn test_l_brackets() {
        let author = Author::create("nick!ident@host", stub_color);
        let result = author.l("Label");
        assert_eq!(result, "\x0314[\x0304Label\x0314]");
    }

    #[test]
    fn test_p_parens() {
        let author = Author::create("nick!ident@host", stub_color);
        let result = author.p("Param");
        assert_eq!(result, "\x0314(\x0304Param\x0314)");
    }

    #[test]
    fn test_c1_c2_accept_non_str() {
        let author = Author::create("nick!ident@host", stub_color);
        assert_eq!(author.c1(42), "\x031442");
        assert_eq!(author.c2(3.14), "\x03043.14");
    }

    // Note: set_colors/clear_colors require a database connection and
    // are tested via integration tests rather than unit tests.
}
