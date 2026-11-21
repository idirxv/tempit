//! Pure-Rust human-readable byte size and relative time formatters.

use chrono::Duration;

/// Format a byte count using binary units (`0 Bytes`, `1 Byte`, `1.5 KiB`, ...).
pub fn format_size_binary(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];
    if bytes == 1 {
        return "1 Byte".to_string();
    }
    if bytes < 1024 {
        return format!("{bytes} Bytes");
    }
    let mut val = bytes as f64 / 1024.0;
    let mut idx = 0;
    while val >= 1024.0 && idx + 1 < UNITS.len() {
        val /= 1024.0;
        idx += 1;
    }
    format!("{val:.1} {}", UNITS[idx])
}

/// Format a duration as a human-readable relative age (`now`, `a minute ago`, `3 hours ago`, ...).
pub fn format_natural_time(delta: Duration) -> String {
    let secs = delta.num_seconds();
    if secs <= 0 {
        return "now".to_string();
    }
    if secs == 1 {
        return "a second ago".to_string();
    }
    if secs < 60 {
        return format!("{secs} seconds ago");
    }
    let mins = secs / 60;
    if mins == 1 {
        return "a minute ago".to_string();
    }
    if mins < 60 {
        return format!("{mins} minutes ago");
    }
    let hours = secs / 3600;
    if hours == 1 {
        return "an hour ago".to_string();
    }
    if hours < 24 {
        return format!("{hours} hours ago");
    }
    let days = secs / 86_400;
    if days == 1 {
        return "a day ago".to_string();
    }
    if days < 30 {
        return format!("{days} days ago");
    }
    let months = days / 30;
    if months == 1 {
        return "a month ago".to_string();
    }
    if days < 365 {
        return format!("{months} months ago");
    }
    let years = days / 365;
    if years == 1 {
        "a year ago".to_string()
    } else {
        format!("{years} years ago")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_binary_formatting() {
        assert_eq!(format_size_binary(0), "0 Bytes");
        assert_eq!(format_size_binary(1), "1 Byte");
        assert_eq!(format_size_binary(512), "512 Bytes");
        assert_eq!(format_size_binary(1024), "1.0 KiB");
        assert_eq!(format_size_binary(1536), "1.5 KiB");
        assert_eq!(format_size_binary(1024 * 1024), "1.0 MiB");
    }

    #[test]
    fn natural_time_formatting() {
        assert_eq!(format_natural_time(Duration::seconds(0)), "now");
        assert_eq!(format_natural_time(Duration::seconds(1)), "a second ago");
        assert_eq!(format_natural_time(Duration::seconds(42)), "42 seconds ago");
        assert_eq!(format_natural_time(Duration::seconds(65)), "a minute ago");
        assert_eq!(format_natural_time(Duration::minutes(5)), "5 minutes ago");
        assert_eq!(format_natural_time(Duration::hours(1)), "an hour ago");
        assert_eq!(format_natural_time(Duration::hours(3)), "3 hours ago");
        assert_eq!(format_natural_time(Duration::days(1)), "a day ago");
        assert_eq!(format_natural_time(Duration::days(4)), "4 days ago");
        assert_eq!(format_natural_time(Duration::days(40)), "a month ago");
        assert_eq!(format_natural_time(Duration::days(400)), "a year ago");
    }
}
