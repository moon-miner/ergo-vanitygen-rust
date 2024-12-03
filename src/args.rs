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

    /// Pattern to look for in addresses
    #[arg(short = 'p', long = "pattern")]
    pub pattern: String,

    /// Generate 12-word seed phrases (default is 24)
    #[arg(long = "w12")]
    pub twelve_words: bool,
}

impl Args {
    pub fn word_count(&self) -> usize {
        if self.twelve_words { 12 } else { 24 }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.start {
            let first_char = self.pattern.chars().next().ok_or("Pattern cannot be empty")?;
            if !['e', 'f', 'g', 'h', 'i'].contains(&first_char) {
                return Err(format!(
                    "When using -s/--start, pattern must start with one of: e, f, g, h, i\n\
                     This is because Ergo P2PK addresses always start with '9' followed by one of these letters.\n\
                     Your pattern '{}' starts with '{}' which will never match.\n\
                     Consider using -e/--end for end matching, or remove -s for anywhere in the address.",
                    self.pattern, first_char
                ));
            }
        }
        Ok(())
    }

    pub fn matcher(&self) -> Box<dyn Fn(&str) -> bool + Send + Sync + '_> {
        let pattern = if self.exact {
            self.pattern.clone()
        } else {
            self.pattern.to_lowercase()
        };
        
        if self.start {
            Box::new(move |addr: &str| {
                if addr.len() <= 1 {
                    return false;
                }
                if self.exact {
                    addr[1..].starts_with(&pattern)
                } else {
                    addr[1..].to_lowercase().starts_with(&pattern.to_lowercase())
                }
            })
        } else {
            Box::new(move |addr: &str| {
                if self.exact {
                    addr.ends_with(&pattern)
                } else {
                    addr.to_lowercase().ends_with(&pattern.to_lowercase())
                }
            })
        }
    }
}