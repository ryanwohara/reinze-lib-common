use anyhow::{bail, Context, Result};
use mysql::prelude::Queryable;
use mysql::{params, from_row};
use regex::Regex;

use crate::database;

/// Parse a duration string like "3d", "12h", "1w", "2w3d" into total hours.
/// Supported units: h (hours), d (days = 24h), w (weeks = 168h). Combinable.
pub fn parse_duration(s: &str) -> Result<u64> {
    let re = Regex::new(r"^(?:(\d+)w)?(?:(\d+)d)?(?:(\d+)h)?$").unwrap();
    let caps = re
        .captures(s)
        .context("invalid duration format, use e.g. @3d, @1w, @12h, @2w3d")?;

    let weeks: u64 = caps.get(1).map_or(0, |m| m.as_str().parse().unwrap_or(0));
    let days: u64 = caps.get(2).map_or(0, |m| m.as_str().parse().unwrap_or(0));
    let hours: u64 = caps.get(3).map_or(0, |m| m.as_str().parse().unwrap_or(0));

    let total = weeks * 168 + days * 24 + hours;
    if total == 0 {
        bail!("duration must be greater than 0");
    }
    Ok(total)
}

/// Store a snapshot of raw hiscores data for a player.
pub fn save_snapshot(game: &str, mode: &str, rsn: &str, data: &str) -> Result<()> {
    let mut conn = database::connect()
        .map_err(|e| anyhow::anyhow!("database connection failed: {}", e))?;

    conn.exec_drop(
        "INSERT INTO hiscores_snapshots (game, mode, rsn, snapshot_at, data) VALUES (:game, :mode, :rsn, NOW(), :data)",
        params! { "game" => game, "mode" => mode, "rsn" => rsn, "data" => data },
    )
    .context("failed to insert snapshot")?;

    Ok(())
}

/// Retrieve the most recent snapshot at least `hours_ago` hours old.
pub fn get_snapshot(
    game: &str,
    mode: &str,
    rsn: &str,
    hours_ago: u64,
) -> Result<Option<String>> {
    let mut conn = database::connect()
        .map_err(|e| anyhow::anyhow!("database connection failed: {}", e))?;

    let result: Option<String> = conn
        .exec_first(
            "SELECT data FROM hiscores_snapshots WHERE game = :game AND mode = :mode AND rsn = :rsn AND snapshot_at <= DATE_SUB(NOW(), INTERVAL :hours HOUR) ORDER BY snapshot_at DESC LIMIT 1",
            params! { "game" => game, "mode" => mode, "rsn" => rsn, "hours" => hours_ago },
        )
        .context("failed to query snapshot")?;

    Ok(result)
}

/// Get all distinct RSNs tracked for a given game (for scheduled snapshots).
pub fn get_tracked_players(game: &str) -> Result<Vec<String>> {
    let mut conn = database::connect()
        .map_err(|e| anyhow::anyhow!("database connection failed: {}", e))?;

    let rows: Vec<mysql::Row> = conn
        .exec(
            "SELECT DISTINCT rsn FROM hiscores_snapshots WHERE game = :game",
            params! { "game" => game },
        )
        .context("failed to query tracked players")?;

    Ok(rows.into_iter().map(|r| from_row(r)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hours() {
        assert_eq!(parse_duration("12h").unwrap(), 12);
    }

    #[test]
    fn test_parse_days() {
        assert_eq!(parse_duration("3d").unwrap(), 72);
    }

    #[test]
    fn test_parse_weeks() {
        assert_eq!(parse_duration("1w").unwrap(), 168);
    }

    #[test]
    fn test_parse_combined() {
        assert_eq!(parse_duration("2w3d").unwrap(), 408);
    }

    #[test]
    fn test_parse_all_units() {
        assert_eq!(parse_duration("1w1d1h").unwrap(), 193);
    }

    #[test]
    fn test_parse_zero() {
        assert!(parse_duration("0h").is_err());
    }

    #[test]
    fn test_parse_empty() {
        assert!(parse_duration("").is_err());
    }

    #[test]
    fn test_parse_invalid() {
        assert!(parse_duration("abc").is_err());
        assert!(parse_duration("10m").is_err());
        assert!(parse_duration("5s").is_err());
    }
}
