use crate::{database, Colors};
use moka::future::Cache;
use mysql::params;
use mysql::prelude::Queryable;
use once_cell::sync::Lazy;
use std::time::Duration;

static CACHE: Lazy<Cache<String, Colors>> = Lazy::new(|| {
    Cache::builder()
        .max_capacity(50_000)
        .time_to_live(Duration::from_secs(900))
        .build()
});

pub async fn get(author_host: String) -> Colors {
    CACHE
        .get_with(author_host.to_string(), async move {
            let mut conn = match database::connect() {
                Ok(conn) => conn,
                Err(e) => {
                    println!("Error connecting to database: {}", e);
                    return Colors::default();
                }
            };

            let colors: Vec<String> = match conn.exec(
                "SELECT color FROM colors WHERE host = :author_host ORDER BY index ASC",
                params! { author_host },
            ) {
                Ok(color) => color,
                Err(e) => {
                    println!("Error getting rsn: {}", e);
                    return Colors::default();
                }
            };

            let color1 = Colors::color1();
            let color2 = Colors::color2();

            let c1 = colors.get(0).unwrap_or(&color1).to_string();
            let c2 = colors.get(1).unwrap_or(&color2).to_string();

            Colors { c1, c2 }
        })
        .await
}

#[allow(dead_code)]
pub async fn set(author_host: String, color: Colors) {
    CACHE.insert(author_host, color).await;
}

#[allow(dead_code)]
pub async fn invalidate(author_host: String) {
    CACHE.invalidate(&author_host).await;
}
