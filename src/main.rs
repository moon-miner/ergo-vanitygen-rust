mod args;
mod utils;
mod address_processor;

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
    
    println!(
        "Looking for {} addresses matching {} patterns {} {} {}",
        args.num_results,
        args.patterns.len(),
        if args.start { "starting with" } else { "ending with" },
        if args.exact { "exactly" } else { "" },
        args.patterns.join(", ")
    );
    println!("Using {}-word seed phrases", args.word_count());

    let processor = AddressProcessor::new();
    let matcher = args.matcher();
    let results = processor.find_matches(matcher, args.word_count(), args.num_results);

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