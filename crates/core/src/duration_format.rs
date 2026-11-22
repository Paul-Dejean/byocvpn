use std::time::Duration;

const SECONDS_PER_MINUTE: u64 = 60;
const SECONDS_PER_HOUR: u64 = 60 * SECONDS_PER_MINUTE;
const SECONDS_PER_DAY: u64 = 24 * SECONDS_PER_HOUR;

pub fn format_duration_compact(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let days = total_seconds / SECONDS_PER_DAY;
    let hours = (total_seconds % SECONDS_PER_DAY) / SECONDS_PER_HOUR;
    let minutes = (total_seconds % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE;
    let seconds = total_seconds % SECONDS_PER_MINUTE;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{days}d"));
    }
    if hours > 0 {
        parts.push(format!("{hours}h"));
    }
    if minutes > 0 {
        parts.push(format!("{minutes}m"));
    }
    if seconds > 0 || parts.is_empty() {
        parts.push(format!("{seconds}s"));
    }
    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_zero_as_seconds() {
        assert_eq!(format_duration_compact(Duration::ZERO), "0s");
    }

    #[test]
    fn skips_empty_units() {
        assert_eq!(
            format_duration_compact(Duration::from_secs(SECONDS_PER_HOUR * 2 + 3)),
            "2h 3s"
        );
    }

    #[test]
    fn formats_every_unit() {
        let duration = Duration::from_secs(SECONDS_PER_DAY + SECONDS_PER_HOUR * 5 + 7 * 60 + 9);
        assert_eq!(format_duration_compact(duration), "1d 5h 7m 9s");
    }
}
