pub struct PatternEstimate {
    pub attempts_needed: f64,
    pub time_at_min: f64,
    pub time_at_max: f64,
}

pub fn estimate_pattern(pattern: &str, is_start: bool) -> PatternEstimate {
    let pattern_length = pattern.len() as f64;
    
    // Calculate probability based on pattern location
    let attempts = if is_start {
        // For start patterns:
        // - Second character must be one of [e,f,g,h,i] (5 possibilities)
        // - Remaining characters are Base58 (58 possibilities)
        5.0 * 58.0f64.powf(pattern_length - 1.0)
    } else {
        // For end/anywhere patterns:
        // - Each position has 58 possibilities
        // - But we can start matching at any position
        // - Average address length is ~40 characters
        let avg_addr_length = 40.0;
        let positions = avg_addr_length - pattern_length + 1.0;
        58.0f64.powf(pattern_length) / positions
    };

    // Add some margin for real-world conditions
    let adjusted_attempts = attempts * 1.2; // 20% safety margin

    // Calculate time estimates using more conservative speeds
    let min_speed = 6_000.0;  // More conservative estimate
    let max_speed = 12_000.0; // More realistic max speed

    PatternEstimate {
        attempts_needed: adjusted_attempts,
        time_at_min: adjusted_attempts / min_speed,
        time_at_max: adjusted_attempts / max_speed,
    }
}

pub fn format_time(seconds: f64) -> String {
    if seconds < 1.0 {
        format!("less than a second")
    } else if seconds < 60.0 {
        format!("{:.1} seconds", seconds)
    } else if seconds < 3600.0 {
        format!("{:.1} minutes", seconds / 60.0)
    } else if seconds < 86400.0 {
        format!("{:.1} hours", seconds / 3600.0)
    } else {
        format!("{:.1} days", seconds / 86400.0)
    }
}

pub fn print_estimate(pattern: &str, is_start: bool) {
    let estimate = estimate_pattern(pattern, is_start);
    
    println!("\nPattern: \"{}\"", pattern);
    println!("Estimated attempts needed: {:.0}", estimate.attempts_needed);
    println!("Estimated time to find:");
    println!("  At 6,000 addr/s: {}", format_time(estimate.time_at_min));
    println!("  At 12,000 addr/s: {}", format_time(estimate.time_at_max));
} 