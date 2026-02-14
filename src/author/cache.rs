use crate::{database, Colors};
use moka::future::Cache;
use mysql::params;
use mysql::prelude::{Queryable, TextQuery};
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

pub async fn set(author_host: String, colors: Colors) {
    let c1 = colors.c1.to_string();
    let c2 = colors.c2.to_string();

    CACHE.insert(author_host.to_string(), colors).await;

    let mut conn = match database::connect() {
        Ok(conn) => conn,
        Err(e) => {
            println!("Error connecting to database: {}", e);
            return;
        }
    };

    let statement = match conn.exec_first::<(String, String), &str, mysql::Params>(
        "SELECT color1, color2 FROM colors WHERE host = :author_host",
        params! { "author_host" => author_host.to_string() },
    ) {
        Ok(Some(_)) => "UPDATE colors SET color1 = :c1, color2 = :c2 WHERE host = :author_host",
        Ok(None) | Err(_) => {
            "INSERT INTO colors (host, color1, color2) values(:author_host, :c1, :c2)"
        }
    }
    .to_string();

    let _ = conn.exec::<bool, &str, mysql::Params>(&statement, params! { author_host, c1, c2 });
}
