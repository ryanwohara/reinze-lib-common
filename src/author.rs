mod cache;

use crate::{c1, c2, Colors};

pub struct Author {
    pub nick: String,
    pub host: String,
    #[allow(dead_code)]
    pub ident: String,
    #[allow(dead_code)]
    pub address: String,
    pub full: String,
}

impl Author {
    pub fn create<T>(a: T) -> Self
    where
        T: ToString,
    {
        let author = a.to_string();
        let (nick, mut host) = author.split_once("!").unwrap_or(("", &author));
        if host.starts_with("~") {
            host = host.split_once("~").unwrap_or(("", host)).1;
        }

        let (ident, address) = host.split_once("@").unwrap_or(("", &host));

        Self {
            nick: nick.to_string(),
            host: host.to_string(),
            ident: ident.to_string(),
            address: address.to_string(),
            full: author.to_string(),
        }
    }

    pub fn c1<T>(&self, s: T) -> String
    where
        T: ToString,
    {
        c1(s)
    }

    pub fn c2<T>(&self, s: T) -> String
    where
        T: ToString,
    {
        c2(s)
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

    pub async fn get_colors(&self) -> Colors {
        cache::get(self.host.to_string()).await
    }

    pub async fn set_colors(&self, colors: Colors) {
        cache::set(self.host.to_string(), colors).await
    }
}
