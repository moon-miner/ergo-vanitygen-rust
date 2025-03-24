use clap::Parser;
use crate::matcher::PatternMatcher;

/// A high-performance vanity address generator for the Ergo blockchain
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Pattern(s) to search for, comma-separated for multiple patterns
    #[arg(short, long, value_delimiter = ',')]
    pub patterns: Vec<String>,

    /// Match at start of address only (after the first '9')
    #[arg(short, long)]
    pub start: bool,

    /// Match at end of address only
    #[arg(short, long)]
    pub end: bool,

    /// Case-sensitive matching (default: case-insensitive)
    #[arg(short = 'm', long = "matchCase")]
    pub case_sensitive: bool,

    /// Generate 12-word seed phrases (default is 24)
    #[arg(long = "w12")]
    pub twelve_word: bool,

    /// Generate 15-word seed phrases (default is 24)
    #[arg(long = "w15")]
    pub fifteen_word: bool,

    /// Generate random seed phrases using all supported lengths (12, 15, 24 words)
    #[arg(long = "wany")]
    pub any_word_length: bool,

    /// Number of addresses to check per seed (default: 1)
    #[arg(short, long, default_value_t = 1)]
    pub addresses_per_seed: u32,

    /// Number of matches to find (default: 1)
    #[arg(short, long = "num", default_value_t = 1)]
    pub num: usize,

    /// Try to find equal matches for all patterns (longer search times)
    #[arg(long)]
    pub balanced: bool,

    /// Estimate difficulty and time for the given pattern
    #[arg(long)]
    pub estimate: bool,

    /// Disable GUI (use command-line only)
    #[arg(long = "no-gui")]
    pub no_gui: bool,
}

impl Args {
    /// Returns the seed word count based on the provided CLI flags.
    pub fn word_count(&self) -> usize {
        if self.any_word_length {
            0 // Special value: use random word count (12, 15, or 24)
        } else if self.twelve_word {
            12
        } else if self.fifteen_word {
            15
        } else {
            24 // Default
        }
    }

    /// Validates the arguments by delegating to the pattern matcher validation logic.
    pub fn validate(&self) -> Result<(), String> {
        // Check if patterns are provided when running in CLI mode
        if self.patterns.is_empty() {
            return Err("At least one pattern must be specified when running in command-line mode".to_string());
        }
        
        self.create_matcher().validate()
    }

    /// Creates a new PatternMatcher based on the provided CLI arguments.
    pub fn create_matcher(&self) -> PatternMatcher {
        PatternMatcher::new(
            self.patterns.clone(),
            self.case_sensitive,
            self.start,
            self.end,
        )
    }
}
