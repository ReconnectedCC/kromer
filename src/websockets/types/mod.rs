pub mod common;
pub mod server;

use chrono::Utc;

#[inline]
pub fn convert_to_iso_string(timestamp: chrono::DateTime<Utc>) -> String {
    timestamp.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}
