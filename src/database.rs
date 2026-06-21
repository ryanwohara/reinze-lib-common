use dotenv::dotenv;
use mysql::*;

/// Opens a fresh connection per call.
///
/// These plugins are loaded behind `dlopen`/`dlclose` once per command, and Rust
/// never drops `static`s on unload — so a long-lived static `Pool` leaked its
/// sockets every command (eventually "Too many open files"). A standalone `Conn`
/// is closed by its `Drop` when the command finishes, before the library unloads.
///
/// `Opts::from_url` is used (rather than the `From<&str>` conversion, which
/// panics on a bad URL) so misconfiguration returns an `Err` instead of
/// unwinding across the plugin's FFI boundary.
pub fn connect() -> std::result::Result<Conn, Error> {
    let opts = Opts::from_url(&get_connection_string())?;
    Conn::new(opts)
}

fn get_connection_string() -> String {
    dotenv().ok();

    // Missing vars default to empty rather than panicking; the resulting
    // connection attempt then fails cleanly and is reported as an `Err`.
    let var = |name: &str| std::env::var(name).unwrap_or_default();
    let host = var("DB_HOST");
    let port = var("DB_PORT");
    let user = var("DB_USER");
    let pass = var("DB_PASS");
    let db = var("DB_NAME");

    format!("mysql://{user}:{pass}@{host}:{port}/{db}")
}
