use crate::{database, ColorResult, Colors};
use arc_swap::ArcSwap;
use mysql::params;
use mysql::prelude::Queryable;
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::{Arc, OnceLock};

type ColorMap = HashMap<String, Colors>;
type ColorCache = Arc<ArcSwap<ColorMap>>;

static COLOR_CACHE: OnceLock<ColorCache> = OnceLock::new();

pub fn init() {
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
            log::error!("Error connecting to database: {}", e);
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
        Ok(None) => Colors::default(),
        Err(e) => {
            log::error!("Error querying database: {}", e);
            Colors::default()
        }
    }
}

pub fn upsert_color(author_host: String, color: Colors) {
    let cache = COLOR_CACHE.get().expect("COLOR_CACHE not initialized");

    cache.rcu(|current| {
        let mut new_map = (**current).clone();
        new_map.insert(author_host.clone(), color.clone());
        Arc::new(new_map)
    });
}

pub fn set(author_host: String, colors: Colors) {
    upsert_color(author_host.clone(), colors.clone());

    let mut conn = match database::connect() {
        Ok(conn) => conn,
        Err(e) => {
            log::error!("Error connecting to database: {}", e);
            return;
        }
    };

    let _ = conn.exec::<bool, &str, mysql::Params>(
        "INSERT INTO colors (host, color1, color2) VALUES (:author_host, :c1, :c2) \
         ON DUPLICATE KEY UPDATE color1 = :c1, color2 = :c2",
        params! { "author_host" => author_host, "c1" => colors.c1, "c2" => colors.c2 },
    );
}

/// Cache-only upsert + read for testing without DB.
/// Uses the already-initialized COLOR_CACHE.
#[cfg(test)]
fn cache_get(author_host: &str) -> Option<Colors> {
    let cache = COLOR_CACHE.get()?;
    let map = cache.load();
    map.get(author_host).cloned()
}

pub extern "C" fn color_ffi(host: *const c_char, to_store: *const c_char) -> ColorResult {
    let hostname = unsafe { CStr::from_ptr(host) }.to_str().unwrap_or_default();
    let colors = unsafe { CStr::from_ptr(to_store) }.to_str().unwrap_or_default();

    if colors.is_empty() {
        let colors = get(hostname);
        ColorResult::from(&colors)
    } else {
        let Some((color1, color2)) = colors.split_once(",") else {
            log::error!("color_ffi: invalid color format (expected 'c1,c2'): {:?}", colors);
            return ColorResult::default();
        };

        let colors = Colors {
            c1: color1.to_string(),
            c2: color2.to_string(),
        };

        set(hostname.to_string(), colors.clone());

        ColorResult::from(&colors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    // Tests run in the same process, and OnceLock can only be set once.
    // We use a single test function for cache operations to avoid init conflicts.
    #[test]
    fn test_cache_upsert_and_read() {
        // Init is idempotent within a single test run because OnceLock.
        // If another test already called init(), this will panic — so we
        // guard with try_init.
        let _ = COLOR_CACHE.set(Arc::new(ArcSwap::from_pointee(HashMap::new())));

        // Empty cache returns None
        assert!(cache_get("unknown@host").is_none());

        // Insert a color
        upsert_color(
            "user@host.com".to_string(),
            Colors {
                c1: "03".to_string(),
                c2: "07".to_string(),
            },
        );

        let cached = cache_get("user@host.com").unwrap();
        assert_eq!(cached.c1, "03");
        assert_eq!(cached.c2, "07");

        // Overwrite
        upsert_color(
            "user@host.com".to_string(),
            Colors {
                c1: "05".to_string(),
                c2: "09".to_string(),
            },
        );

        let cached = cache_get("user@host.com").unwrap();
        assert_eq!(cached.c1, "05");
        assert_eq!(cached.c2, "09");
    }

    #[test]
    fn test_cache_multiple_hosts() {
        let _ = COLOR_CACHE.set(Arc::new(ArcSwap::from_pointee(HashMap::new())));

        upsert_color(
            "alice@a.com".to_string(),
            Colors {
                c1: "01".to_string(),
                c2: "02".to_string(),
            },
        );
        upsert_color(
            "bob@b.com".to_string(),
            Colors {
                c1: "03".to_string(),
                c2: "04".to_string(),
            },
        );

        let alice = cache_get("alice@a.com").unwrap();
        let bob = cache_get("bob@b.com").unwrap();

        assert_eq!(alice.c1, "01");
        assert_eq!(bob.c1, "03");
        // Both should still be present
        assert!(cache_get("alice@a.com").is_some());
        assert!(cache_get("bob@b.com").is_some());
    }

    #[test]
    fn test_cache_rcu_does_not_lose_entries() {
        let _ = COLOR_CACHE.set(Arc::new(ArcSwap::from_pointee(HashMap::new())));

        // Insert several entries sequentially
        for i in 0..10 {
            upsert_color(
                format!("host{}", i),
                Colors {
                    c1: format!("{:02}", i),
                    c2: format!("{:02}", i + 10),
                },
            );
        }

        // All entries should be present
        for i in 0..10 {
            let cached = cache_get(&format!("host{}", i)).unwrap();
            assert_eq!(cached.c1, format!("{:02}", i));
        }
    }

    #[test]
    fn test_color_result_from_colors() {
        let colors = Colors {
            c1: "14".to_string(),
            c2: "04".to_string(),
        };
        let result = ColorResult::from(&colors);

        // The pointers should be valid C strings
        unsafe {
            let c1 = CStr::from_ptr(result.c1).to_str().unwrap();
            let c2 = CStr::from_ptr(result.c2).to_str().unwrap();
            assert_eq!(c1, "14");
            assert_eq!(c2, "04");

            // Clean up leaked pointers
            _ = CString::from_raw(result.c1 as *mut c_char);
            _ = CString::from_raw(result.c2 as *mut c_char);
        }
    }

    #[test]
    fn test_color_result_default() {
        let result = ColorResult::default();

        unsafe {
            let c1 = CStr::from_ptr(result.c1).to_str().unwrap();
            let c2 = CStr::from_ptr(result.c2).to_str().unwrap();
            assert_eq!(c1, "14");
            assert_eq!(c2, "04");

            _ = CString::from_raw(result.c1 as *mut c_char);
            _ = CString::from_raw(result.c2 as *mut c_char);
        }
    }
}
