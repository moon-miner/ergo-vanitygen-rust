/// Module for address pattern matching functionality
/// Extracts matcher logic from args.rs and address_processor.rs

pub struct PatternMatcher {
    patterns: Vec<String>,
    case_sensitive: bool,
    start: bool,
    end: bool,
}

impl PatternMatcher {
    pub fn new(patterns: Vec<String>, case_sensitive: bool, start: bool, end: bool) -> Self {
        PatternMatcher {
            patterns: if case_sensitive {
                patterns
            } else {
                patterns.iter().map(|p| p.to_lowercase()).collect()
            },
            case_sensitive,
            start,
            end,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.patterns.is_empty() {
            return Err("At least one pattern must be provided".to_string());
        }

        if self.start {
            for pattern in &self.patterns {
                let first_char = pattern.chars().next().ok_or("Pattern cannot be empty")?;
                if !['e', 'f', 'g', 'h', 'i'].contains(&first_char) {
                    return Err(format!(
                        "When using start matching, patterns must start with one of: e, f, g, h, i\n\
                         This is because Ergo P2PK addresses always start with '9' followed by one of these letters.\n\
                         Your pattern '{}' starts with '{}' which will never match.\n\
                         Consider using end matching, or use anywhere matching instead.",
                        pattern, first_char
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn is_match(&self, address: &str) -> Option<String> {
        if self.start {
            self.match_start(address)
        } else if self.end {
            self.match_end(address)
        } else {
            self.match_anywhere(address)
        }
    }

    fn match_start(&self, address: &str) -> Option<String> {
        if address.len() <= 1 {
            return None;
        }
        let addr_to_check = if self.case_sensitive {
            &address[1..]
        } else {
            &address[1..].to_lowercase()
        };
        
        for pattern in &self.patterns {
            if addr_to_check.starts_with(pattern) {
                return Some(pattern.clone());
            }
        }
        None
    }

    fn match_end(&self, address: &str) -> Option<String> {
        let addr_to_check = if self.case_sensitive {
            address.to_string()
        } else {
            address.to_lowercase()
        };
        
        for pattern in &self.patterns {
            if addr_to_check.ends_with(pattern) {
                return Some(pattern.clone());
            }
        }
        None
    }

    fn match_anywhere(&self, address: &str) -> Option<String> {
        let addr_to_check = if self.case_sensitive {
            address.to_string()
        } else {
            address.to_lowercase()
        };
        
        for pattern in &self.patterns {
            if addr_to_check.contains(pattern) {
                return Some(pattern.clone());
            }
        }
        None
    }

    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }
} 