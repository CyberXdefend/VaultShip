use anyhow::Result;
use chrono::{DateTime, Utc};

pub fn next_heartbeat_at(interval_minutes: i64) -> Result<DateTime<Utc>> {
    Ok(Utc::now() + chrono::Duration::minutes(interval_minutes))
}
