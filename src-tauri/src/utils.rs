use crate::constants::AVERAGE_DELAY_SECS;

/// Calculate ETA string from remaining count
pub fn calculate_eta(remaining: u32) -> String {
    let eta_seconds = remaining as u64 * AVERAGE_DELAY_SECS;
    if eta_seconds > 3600 {
        format!("{}h {}m", eta_seconds / 3600, (eta_seconds % 3600) / 60)
    } else if eta_seconds > 60 {
        format!("{}m", eta_seconds / 60)
    } else {
        format!("{}s", eta_seconds)
    }
}

/// Format timestamp for display
pub fn format_timestamp() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_eta() {
        assert_eq!(calculate_eta(0), "0s");
        assert_eq!(calculate_eta(1), "45s");
        assert_eq!(calculate_eta(2), "1m");
        assert_eq!(calculate_eta(80), "1h 0m");
    }
}
