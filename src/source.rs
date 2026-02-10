use crate::author::Author;

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
}
