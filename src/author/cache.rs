use crate::{database, ColorResult, Colors};
use arc_swap::ArcSwap;
use mysql::params;
use mysql::prelude::Queryable;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
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

pub fn get<T>(author_host: T) -> Colors
where
    T: ToString,
{
    let cache = COLOR_CACHE.get().expect("Cache not initialized");

    let map = cache.load();

    match map.get(&author_host.to_string()) {
        Some(colors) => colors.to_owned(),
        None => {
            let colors = get_from_db(author_host.to_string());
            upsert_color(author_host.to_string(), colors.clone());

            colors
        }
    }
}

fn get_from_db(author_host: String) -> Colors {
    let mut conn = match database::connect() {
        Ok(conn) => conn,
        Err(e) => {
            println!("Error connecting to database: {}", e);
            return Colors::default();
        }
    };

    match conn.exec_first::<(String, String), &str, mysql::Params>(
        "SELECT color1, color2 FROM colors WHERE host = :author_host",
        params! { author_host },
    ) {
        Ok(Some(colors)) => Colors {
            c1: colors.0,
            c2: colors.1,
        },
        Ok(None) | Err(_) => Colors::default(),
    }
}

pub fn upsert_color(author_host: String, color: Colors) {
    let cache = COLOR_CACHE.get().expect("COLOR_CACHE not initialized");

    let current = cache.load();
    let mut new_map = (**current).clone();

    new_map.insert(author_host, color);

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

pub extern "C" fn color_ffi(host: *const c_char, to_store: *const c_char) -> ColorResult {
    let hostname = unsafe { CStr::from_ptr(host) }.to_str().unwrap();
    let colors = unsafe { CStr::from_ptr(to_store) }.to_str().unwrap();

    if colors.len() == 0 {
        let colors = get(hostname);

        let c1 = CString::new(colors.c1.to_string()).unwrap().into_raw();
        let c2 = CString::new(colors.c2.to_string()).unwrap().into_raw();

        let result = ColorResult { c1, c2 };

        unsafe {
            _ = CStr::from_ptr(c1);
            _ = CStr::from_ptr(c2);
        }

        result
    } else {
        let split = colors.split_once(",").unwrap();

        let color1 = split.0.to_string();
        let color2 = split.1.to_string();

        set(
            hostname.to_string(),
            Colors {
                c1: color1.to_string(),
                c2: color2.to_string(),
            },
        );

        let c1 = CString::new(color1).unwrap().into_raw();
        let c2 = CString::new(color2).unwrap().into_raw();

        let result = ColorResult { c1, c2 };

        unsafe {
            _ = CStr::from_ptr(c1);
            _ = CStr::from_ptr(c2);
        }

        result
    }
}
