use std::sync::atomic::{AtomicBool, Ordering};
use clap::Parser;
use std::time::Instant;

// Define modules
mod args;
mod address_processor;
mod progress;
mod utils;
mod matcher;
mod estimator;
mod paper_wallet;
mod crypto;

#[cfg(feature = "gui")]
mod gui;

use args::Args;

fn main() {
    let args = Args::parse();

    // Initialize hardware acceleration if available
    if cfg!(feature = "hw_accel") {
        crypto::get_context().log_features();
    }

    // GUI Mode check - if no explicit patterns provided and no-gui isn't specified,
    // we default to GUI mode if the feature is enabled
    #[cfg(feature = "gui")]
    {
        let should_launch_gui = args.patterns.is_empty() && !args.no_gui && !args.estimate;
        if should_launch_gui {
            if let Err(e) = gui::run_gui() {
                eprintln!("Error running GUI: {}", e);
                std::process::exit(1);
            }
            return;
        }
    }

    // If estimate flag is set, run the estimation and exit
    if args.estimate {
        if args.patterns.is_empty() {
            eprintln!("Error: Please provide at least one pattern for estimation with --patterns");
            std::process::exit(1);
        }

        let patterns = args.patterns.clone();
        for pattern in patterns {
            estimator::estimate_and_print(&pattern, args.start);
        }
        return;
    }

    // Validate arguments for CLI mode
    if let Err(err) = args.validate() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }

    // Print processing information
    println!(
        "Looking for {} addresses matching {} patterns {}{}",
        args.num,
        args.patterns.len(),
        if args.start { "starting with " } else if args.end { "ending with " } else { "containing " },
        args.patterns.join(", ")
    );
    println!("Using {}-word seed phrases", args.word_count());
    println!("Checking {} addresses per seed", args.addresses_per_seed);

    // Set up processor
    let processor = address_processor::AddressProcessor::new();
    let start_time = Instant::now();

    // Register Ctrl+C handler
    static CANCEL_FLAG: AtomicBool = AtomicBool::new(false);
    ctrlc::set_handler(move || {
        if CANCEL_FLAG.load(Ordering::SeqCst) {
            // Second Ctrl+C, force exit
            std::process::exit(1);
        }
        CANCEL_FLAG.store(true, Ordering::SeqCst);
        eprintln!("\nCtrl+C received, attempting to cancel... Press Ctrl+C again to force exit.");
    }).expect("Error setting Ctrl+C handler");

    // Run the search
    let matcher = args.create_matcher();
    let _results = processor.find_matches(
        matcher,
        args.word_count() as usize,
        args.num,
        args.balanced,
        args.addresses_per_seed
    );

    // If cancelled, print message and exit
    if CANCEL_FLAG.load(Ordering::SeqCst) {
        println!("Search cancelled by user.");
        std::process::exit(1);
    }

    // Get and display performance stats
    let (total_seeds, total_addresses, seed_rate, address_rate, threads) = processor.get_stats();
    println!("\nPerformance Statistics:");
    println!("- Using {} threads", threads);
    println!("- Checked {} seeds", total_seeds);
    println!("- Checked {} addresses", total_addresses);
    println!("- Average speed: {:.0} seeds/second", seed_rate);
    println!("- Average speed: {:.0} addresses/second", address_rate);

    // Display timing
    let duration = start_time.elapsed();
    println!("- Total search time: {:.2} seconds", duration.as_secs_f64());

    // Done
    std::process::exit(0);
}