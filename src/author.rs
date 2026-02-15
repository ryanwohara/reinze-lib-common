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
    pub get_color: extern "C" fn(*const c_char, *const c_char) -> ColorResult,
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
            get_color: f,
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
        format!("{}{}{}", self.c1("["), self.c2(s), self.c1("]"))
    }

    pub fn p<T>(&self, s: T) -> String
    where
        T: ToString,
    {
        format!("{}{}{}", self.c1("("), self.c2(s), self.c1(")"))
    }

    pub unsafe fn colors(&self) -> Colors {
        let host = CString::new(self.host.to_string()).unwrap().into_raw();
        let empty = CString::new("").unwrap().into_raw();

        let results = (self.get_color)(host, empty);

        let c1 = CStr::from_ptr(results.c1).to_string_lossy().into_owned();
        let c2 = CStr::from_ptr(results.c2).to_string_lossy().into_owned();

        _ = CString::from_raw(host);
        _ = CString::from_raw(empty);

        Colors { c1, c2 }
    }

    pub fn set_colors(&self, colors: Colors) {
        cache::set(self.host.to_string(), colors)
    }

    pub fn clear_colors(&self) {
        self.set_colors(Colors::default())
    }
}

fn wrap(s: &str, color: &str) -> String {
    format!("\x03{}{}", color, s)
}
