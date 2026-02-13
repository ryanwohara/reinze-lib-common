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

            let colors: (String, String) = match conn.exec_first(
                "SELECT color1, color2 FROM colors WHERE host = :author_host",
                params! { author_host },
            ) {
                Ok(Some(colors)) => colors,
                Ok(None) => (Colors::color1(), Colors::color2()),
                Err(e) => {
                    println!("Error getting rsn: {}", e);
                    return Colors::default();
                }
            };

            let c1 = colors.0;
            let c2 = colors.1;

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
