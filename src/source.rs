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

    pub async fn get_colors(&self) -> Colors {
        self.author.get_colors().await
    }

    pub async fn set_colors(&self, colors: Colors) {
        self.author.set_colors(colors).await
    }

    pub async fn clear_colors(&self) {
        self.author.clear_colors().await
    }
}
