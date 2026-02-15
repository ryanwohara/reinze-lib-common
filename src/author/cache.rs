use crate::{database, Colors};
use arc_swap::ArcSwap;
use mysql::params;
use mysql::prelude::Queryable;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

type ColorMap = HashMap<String, Colors>;
type ColorCache = Arc<ArcSwap<ColorMap>>;

static COLOR_CACHE: OnceLock<ColorCache> = OnceLock::new();

pub async fn init() {
    let cache = Arc::new(ArcSwap::from_pointee(HashMap::new()));
    COLOR_CACHE
        .set(cache.clone())
        .expect("Color cache failed to initialize");
}

pub fn get(author_host: String) -> Colors {
    let cache = COLOR_CACHE.get().expect("Cache not initialized");

    let map = cache.load();

    map.get(&author_host)
        .unwrap_or(&Colors::default())
        .to_owned()
}

pub fn upsert_color(name: String, color: Colors) {
    let cache = COLOR_CACHE.get().expect("COLOR_CACHE not initialized");

    let current = cache.load();
    let mut new_map = (**current).clone();

    new_map.insert(name, color);

    cache.store(Arc::new(new_map));
}

pub fn set(author_host: String, colors: Colors) {
    let c1 = colors.c1.to_string();
    let c2 = colors.c2.to_string();

    upsert_color(author_host.to_string(), colors);

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
