use clap::Parser;
use crate::matcher::PatternMatcher;

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

    /// Generate 15-word seed phrases (default is 24)
    #[arg(long = "w15")]
    pub fifteen_words: bool,

    /// Generate random seed phrases using all supported lengths (12, 15, 24 words)
    #[arg(long = "wall")]
    pub all_word_lengths: bool,

    /// Number of matching addresses to find (default: 1)
    #[arg(short = 'n', long = "num", default_value = "1")]
    pub num_results: usize,

    /// Number of addresses to check per seed (default: 1)
    #[arg(short = 'i', long = "index", default_value = "1")]
    pub addresses_per_seed: u32,

    /// Try to find matches for all patterns evenly
    #[arg(short = 'b', long = "balanced")]
    pub balanced: bool,

    /// Estimate time to find matches before starting
    #[arg(long = "estimate")]
    pub estimate: bool,
    
    /// Run in CLI mode without launching the GUI
    #[arg(long = "no-gui", hide = true)]
    pub no_gui: bool,
}

impl Args {
    pub fn word_count(&self) -> usize {
        if self.all_word_lengths {
            0 // Special value to indicate all lengths should be used
        } else if self.twelve_words { 
            12 
        } else if self.fifteen_words {
            15
        } else { 
            24 
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        // Create a matcher to use its validation logic
        let matcher = self.create_matcher();
        matcher.validate()
    }

    pub fn create_matcher(&self) -> PatternMatcher {
        PatternMatcher::new(
            self.patterns.clone(),
            self.exact,
            self.start,
            self.end
        )
    }
}