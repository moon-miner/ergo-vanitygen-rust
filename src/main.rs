mod args;
mod utils;
mod address_processor;
mod estimator;
mod matcher;
mod progress;

#[cfg(feature = "gui")]
mod gui;

mod paper_wallet;

use clap::Parser;
use args::Args;
use address_processor::AddressProcessor;

fn main() {
    #[cfg(feature = "gui")]
    {
        // Check for --no-gui flag
        if std::env::args().any(|arg| arg == "--no-gui") {
            run_cli();
        } else {
            // Launch GUI by default
            if let Err(e) = gui::run_gui() {
                eprintln!("Error running GUI: {}", e);
                std::process::exit(1);
            }
        }
    }

    #[cfg(not(feature = "gui"))]
    {
        run_cli();
    }
}

fn run_cli() {
    let args = Args::parse();
    
    // Validate arguments
    if let Err(err) = args.validate() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
    
    // If estimation mode, show estimates and exit
    if args.estimate {
        println!("Difficulty Estimation");
        println!("====================");
        for pattern in &args.patterns {
            estimator::print_estimate(pattern, args.start);
        }
        if args.balanced {
            println!("\nNote: Using balanced mode will increase time as it tries to find all patterns");
        }
        return;
    }
    
    // Create matcher and start search
    let location = if args.start { 
        "starting with" 
    } else if args.end { 
        "ending with" 
    } else { 
        "containing" 
    };
    
    println!(
        "Looking for {} addresses {} {} patterns {} {} {}",
        args.num_results,
        if args.balanced { "balanced across" } else { "matching" },
        args.patterns.len(),
        location,
        if args.exact { "exactly" } else { "" },
        args.patterns.join(", ")
    );
    println!("Using {}-word seed phrases", args.word_count());
    println!("Checking {} addresses per seed", args.addresses_per_seed);

    let processor = AddressProcessor::new();
    let matcher = args.create_matcher();
    let _results = processor.find_matches(
        matcher, 
        args.word_count(), 
        args.num_results,
        args.balanced,
        args.addresses_per_seed
    );

    // Get and display performance stats
    let (total_seeds, total_addresses, seed_rate, address_rate, threads) = processor.get_stats();
    println!("\nPerformance Statistics:");
    println!("- Using {} threads", threads);
    println!("- Checked {} seeds", total_seeds);
    println!("- Checked {} addresses", total_addresses);
    println!("- Average speed: {:.0} seeds/second", seed_rate);
    println!("- Average speed: {:.0} addresses/second", address_rate);
}