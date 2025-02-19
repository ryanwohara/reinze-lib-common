use dotenv::dotenv;
use mysql::{prelude::*, *};
use reqwest;

pub fn connect() -> std::result::Result<PooledConn, Error> {
    get_connection(&get_connection_string())
}

fn get_connection(url: &str) -> std::result::Result<PooledConn, Error> {
    let pool = Pool::new(url)?;

    pool.get_conn()
}

fn get_connection_string() -> String {
    dotenv().ok();

    let host = std::env::var("DB_HOST").expect("DB_HOST must be set");
    let port = std::env::var("DB_PORT").expect("DB_PORT must be set");
    let user = std::env::var("DB_USER").expect("DB_USER must be set");
    let pass = std::env::var("DB_PASS").expect("DB_PASS must be set");
    let db = std::env::var("DB_NAME").expect("DB_NAME must be set");

    format!("mysql://{user}:{pass}@{host}:{port}/{db}")
}
