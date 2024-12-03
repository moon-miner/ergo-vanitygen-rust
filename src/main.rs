mod args;
mod utils;
mod address_processor;
mod estimator;

use clap::Parser;
use args::Args;
use address_processor::AddressProcessor;

fn main() {
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
    
    println!(
        "Looking for {} addresses {} {} patterns {} {} {}",
        args.num_results,
        if args.balanced { "balanced across" } else { "matching" },
        args.patterns.len(),
        if args.start { "starting with" } else { "ending with" },
        if args.exact { "exactly" } else { "" },
        args.patterns.join(", ")
    );
    println!("Using {}-word seed phrases", args.word_count());

    let processor = AddressProcessor::new();
    let matcher = args.matcher();
    let results = processor.find_matches(
        matcher, 
        args.word_count(), 
        args.num_results,
        args.balanced,
        args.patterns.len()
    );

    // Get and display performance stats
    let (total, rate, threads) = processor.get_stats();
    println!("\nPerformance Statistics:");
    println!("- Using {} threads", threads);
    println!("- Checked {} addresses", total);
    println!("- Average speed: {:.0} addresses/second", rate);

    println!(
        "\nFound {} matching addresses:",
        results.len(),
    );

    for (i, (seed, addr, pattern)) in results.iter().enumerate() {
        println!("---------------------------");
        println!("Match {} of {}", i + 1, args.num_results);
        println!("Pattern matched: {}", pattern);
        println!("Seed phrase: {}", seed);
        println!("Address: {}", addr);
        println!("---------------------------");
    }
}