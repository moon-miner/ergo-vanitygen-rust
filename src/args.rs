use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "ergo-vanitygen")]
#[command(about = "Generate vanity Ergo addresses")]
pub struct Args {
    /// Look for pattern at the start of addresses (must start with one of: e, f, g, h, i)
    #[arg(short = 's', long = "start", conflicts_with = "end")]
    pub start: bool,

    /// Look for pattern at the end of addresses
    #[arg(short = 'e', long = "end", conflicts_with = "start")]
    pub end: bool,

    /// Match provided pattern with case sensitivity
    #[arg(short = 'm', long = "matchCase")]
    pub exact: bool,

    /// Patterns to look for in addresses (comma-separated)
    #[arg(short = 'p', long = "pattern", value_delimiter = ',')]
    pub patterns: Vec<String>,

    /// Generate 12-word seed phrases (default is 24)
    #[arg(long = "w12")]
    pub twelve_words: bool,

    /// Number of matching addresses to find (default: 1)
    #[arg(short = 'n', long = "num", default_value = "1")]
    pub num_results: usize,

    /// Try to find matches for all patterns evenly
    #[arg(short = 'b', long = "balanced")]
    pub balanced: bool,

    /// Estimate time to find matches before starting
    #[arg(long = "estimate")]
    pub estimate: bool,
}

impl Args {
    pub fn word_count(&self) -> usize {
        if self.twelve_words { 12 } else { 24 }
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
                        "When using -s/--start, patterns must start with one of: e, f, g, h, i\n\
                         This is because Ergo P2PK addresses always start with '9' followed by one of these letters.\n\
                         Your pattern '{}' starts with '{}' which will never match.\n\
                         Consider using -e/--end for end matching, or remove -s for anywhere in the address.",
                        pattern, first_char
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn matcher(&self) -> Box<dyn Fn(&str) -> Option<String> + Send + Sync + '_> {
        let patterns: Vec<String> = if self.exact {
            self.patterns.clone()
        } else {
            self.patterns.iter().map(|p| p.to_lowercase()).collect()
        };
        
        if self.start {
            Box::new(move |addr: &str| {
                if addr.len() <= 1 {
                    return None;
                }
                let addr_to_check = if self.exact {
                    &addr[1..]
                } else {
                    &addr[1..].to_lowercase()
                };
                
                for pattern in &patterns {
                    if addr_to_check.starts_with(pattern) {
                        return Some(pattern.clone());
                    }
                }
                None
            })
        } else {
            Box::new(move |addr: &str| {
                let addr_to_check = if self.exact {
                    addr.to_string()
                } else {
                    addr.to_lowercase()
                };
                
                for pattern in &patterns {
                    if addr_to_check.ends_with(pattern) {
                        return Some(pattern.clone());
                    }
                }
                None
            })
        }
    }
}