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
        "Looking for {} addresses that {} {} with \"{}\"",
        args.num_results,
        if args.start { "start" } else { "end" },
        if args.exact { "exactly" } else { "" },
        args.pattern
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

    for (i, (seed, addr)) in results.iter().enumerate() {
        println!("---------------------------");
        println!("Match {} of {}", i + 1, args.num_results);
        println!("Seed phrase: {}", seed);
        println!("Address: {}", addr);
        println!("---------------------------");
    }
}