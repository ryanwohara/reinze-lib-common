use crate::author::Author;
use crate::Colors;

pub struct Source {
    pub rsn_n: String,
    pub author: Author,
    pub command: String,
    pub query: String,
}

impl Source {
    pub fn create<T>(rsn_n: T, author: Author, command: T, query: T) -> Self
    where
        T: ToString,
    {
        Self {
            rsn_n: rsn_n.to_string(),
            author,
            command: command.to_string(),
            query: query.to_string(),
        }
    }

    pub fn c1<T>(&self, s: T) -> String
    where
        T: ToString,
    {
        self.author.c1(s)
    }

    pub fn c2<T>(&self, s: T) -> String
    where
        T: ToString,
    {
        self.author.c2(s)
    }

    pub fn l<T>(&self, s: T) -> String
    where
        T: ToString,
    {
        self.author.l(s)
    }

    pub fn p<T>(&self, s: T) -> String
    where
        T: ToString,
    {
        self.author.p(s)
    }

    pub fn get_colors(&self) -> Colors {
        unsafe { self.author.colors() }
    }

    pub fn set_colors(&self, colors: Colors) {
        self.author.set_colors(colors)
    }

    pub fn clear_colors(&self) {
        self.author.clear_colors()
    }

    pub fn not_found(&self, v: Vec<String>) -> String {
        if v.is_empty() {
            self.c2("Not found")
        } else {
            v.join(&self.c1(" | "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ColorResult;

    extern "C" fn stub_color(_host: *const std::os::raw::c_char, _colors: *const std::os::raw::c_char) -> ColorResult {
        ColorResult::default()
    }

    fn make_source(rsn_n: &str, author_str: &str, command: &str, query: &str) -> Source {
        Source::create(rsn_n, Author::create(author_str, stub_color), command, query)
    }

    #[test]
    fn test_create_stores_fields() {
        let source = make_source("2", "nick!ident@host", "stats", "zezima");
        assert_eq!(source.rsn_n, "2");
        assert_eq!(source.command, "stats");
        assert_eq!(source.query, "zezima");
        assert_eq!(source.author.nick, "nick");
    }

    #[test]
    fn test_create_default_rsn() {
        let source = make_source("0", "nick!ident@host", "ge", "dragon bones");
        assert_eq!(source.rsn_n, "0");
    }

    #[test]
    fn test_create_empty_query() {
        let source = make_source("0", "nick!ident@host", "players", "");
        assert_eq!(source.query, "");
    }

    #[test]
    fn test_c1_delegates() {
        let source = make_source("0", "nick!ident@host", "cmd", "q");
        assert_eq!(source.c1("text"), "\x0314text");
    }

    #[test]
    fn test_c2_delegates() {
        let source = make_source("0", "nick!ident@host", "cmd", "q");
        assert_eq!(source.c2("text"), "\x0304text");
    }

    #[test]
    fn test_l_delegates() {
        let source = make_source("0", "nick!ident@host", "cmd", "q");
        assert_eq!(source.l("Stats"), "\x0314[\x0304Stats\x0314]");
    }

    #[test]
    fn test_p_delegates() {
        let source = make_source("0", "nick!ident@host", "cmd", "q");
        assert_eq!(source.p("Info"), "\x0314(\x0304Info\x0314)");
    }

    #[test]
    fn test_get_colors_returns_defaults() {
        let source = make_source("0", "nick!ident@host", "cmd", "q");
        let colors = source.get_colors();
        assert_eq!(colors.c1, "14");
        assert_eq!(colors.c2, "04");
    }

    #[test]
    fn test_create_accepts_to_string_types() {
        // All args must be the same type T: ToString, so we use &str consistently
        let source = Source::create("0", Author::create("nick!i@h", stub_color), "cmd", "q");
        assert_eq!(source.rsn_n, "0");
        // Also works with String
        let source = Source::create(
            String::from("1"),
            Author::create("nick!i@h", stub_color),
            String::from("stats"),
            String::from("query"),
        );
        assert_eq!(source.rsn_n, "1");
        assert_eq!(source.command, "stats");
    }
}
