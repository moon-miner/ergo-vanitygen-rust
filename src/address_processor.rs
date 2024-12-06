use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use rayon::prelude::*;
use indicatif::{ProgressBar, ProgressStyle};
use crate::utils::{generate_mnemonic, generate_addresses};
use std::sync::Mutex;

pub struct AddressProcessor {
    total_seeds: Arc<AtomicUsize>,
    total_addresses: Arc<AtomicUsize>,
    running: Arc<AtomicBool>,
    start_time: Instant,
    thread_count: usize,
    batch_size: usize,
}

impl AddressProcessor {
    pub fn new() -> Self {
        let cpu_count = num_cpus::get();
        let thread_count = cpu_count.max(1);
        
        // Fixed batch size - simple and efficient
        let batch_size = 1000;

        // Configure thread pool
        rayon::ThreadPoolBuilder::new()
            .num_threads(thread_count)
            .build_global()
            .unwrap();

        Self {
            total_seeds: Arc::new(AtomicUsize::new(0)),
            total_addresses: Arc::new(AtomicUsize::new(0)),
            running: Arc::new(AtomicBool::new(true)),
            start_time: Instant::now(),
            thread_count,
            batch_size,
        }
    }

    pub fn find_matches(
        &self,
        matcher: impl Fn(&str) -> Option<String> + Send + Sync,
        word_count: usize,
        num_results: usize,
        balanced: bool,
        pattern_count: usize,
        addresses_per_seed: u32,
    ) -> Vec<(String, String, String, u32)> {
        let progress = Arc::new(ProgressBar::new_spinner());
        progress.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} [{elapsed_precise}] {msg}")
                .unwrap()
        );

        let total_seeds = self.total_seeds.clone();
        let total_addresses = self.total_addresses.clone();
        let running = self.running.clone();
        let progress_clone = progress.clone();
        
        let progress_thread = std::thread::spawn(move || {
            let mut last_seeds = 0;
            let mut last_addresses = 0;
            let mut last_time = Instant::now();
            let mut max_seed_rate = 0f64;
            let mut max_addr_rate = 0f64;
            let mut first_update = true;

            while running.load(Ordering::Relaxed) {
                let current_seeds = total_seeds.load(Ordering::Relaxed);
                let current_addresses = total_addresses.load(Ordering::Relaxed);
                let current_time = Instant::now();
                
                let delta_seeds = current_seeds - last_seeds;
                let delta_addresses = current_addresses - last_addresses;
                let delta_time = current_time.duration_since(last_time).as_secs_f64();
                
                if delta_time >= 0.1 {
                    let seed_rate = delta_seeds as f64 / delta_time;
                    let addr_rate = delta_addresses as f64 / delta_time;
                    
                    if !first_update {
                        max_seed_rate = max_seed_rate.max(seed_rate);
                        max_addr_rate = max_addr_rate.max(addr_rate);
                    }
                    first_update = false;

                    progress_clone.set_message(format!(
                        "Checked {} seeds ({:.0} seeds/s) and {} addresses ({:.0} addr/s)...",
                        current_seeds, seed_rate, current_addresses, addr_rate
                    ));

                    last_seeds = current_seeds;
                    last_addresses = current_addresses;
                    last_time = current_time;
                }

                std::thread::sleep(Duration::from_secs(1));
            }
        });

        let matches: Vec<(String, String, String, u32)> = if balanced {
            let per_pattern = (num_results + pattern_count - 1) / pattern_count;
            let pattern_matches = Arc::new(Mutex::new(std::collections::HashMap::new()));
            let pattern_matches_clone = pattern_matches.clone();
            let found_count = Arc::new(AtomicUsize::new(0));
            let found_count_clone = found_count.clone();

            rayon::iter::repeat(())
                .map_init(
                    || (),
                    |_, _| {
                        let mut batch_results = Vec::new();
                        for _ in 0..self.batch_size {
                            self.total_seeds.fetch_add(1, Ordering::Relaxed);
                            let mnemonic = generate_mnemonic(word_count);
                            let addresses = generate_addresses(&mnemonic, addresses_per_seed);
                            self.total_addresses.fetch_add(addresses.len(), Ordering::Relaxed);
                            
                            for addr_info in addresses {
                                if let Some(pattern) = matcher(&addr_info.address) {
                                    let count = pattern_matches_clone.lock().unwrap()
                                        .get(&pattern).copied().unwrap_or(0);
                                    if count < per_pattern {
                                        batch_results.push((mnemonic.clone(), addr_info.address, pattern.clone(), addr_info.position));
                                    }
                                }
                            }
                        }
                        batch_results
                    }
                )
                .flatten()
                .inspect(|(mnemonic, address, pattern, position)| {
                    let mut map = pattern_matches_clone.lock().unwrap();
                    let current_count = map.entry(pattern.clone()).or_insert(0);
                    *current_count += 1;
                    let total_found = found_count_clone.fetch_add(1, Ordering::Relaxed) + 1;
                    
                    // Display result immediately
                    println!("\n---------------------------");
                    println!("Match {} of {}", total_found, num_results);
                    println!("Pattern matched: {}", pattern);
                    println!("Seed phrase: {}", mnemonic);
                    println!("Address: {}", address);
                    println!("Position: {}", position);
                    println!("---------------------------");
                })
                .take_any(num_results)
                .collect()
        } else {
            let found_count = Arc::new(AtomicUsize::new(0));
            let found_count_clone = found_count.clone();

            rayon::iter::repeat(())
                .map_init(
                    || (),
                    |_, _| {
                        let mut batch_results = Vec::new();
                        for _ in 0..self.batch_size {
                            self.total_seeds.fetch_add(1, Ordering::Relaxed);
                            let mnemonic = generate_mnemonic(word_count);
                            let addresses = generate_addresses(&mnemonic, addresses_per_seed);
                            self.total_addresses.fetch_add(addresses.len(), Ordering::Relaxed);
                            
                            for addr_info in addresses {
                                if let Some(pattern) = matcher(&addr_info.address) {
                                    batch_results.push((mnemonic.clone(), addr_info.address, pattern, addr_info.position));
                                }
                            }
                        }
                        batch_results
                    }
                )
                .flatten()
                .inspect(|(mnemonic, address, pattern, position)| {
                    let total_found = found_count_clone.fetch_add(1, Ordering::Relaxed) + 1;
                    
                    // Display result immediately
                    println!("\n---------------------------");
                    println!("Match {} of {}", total_found, num_results);
                    println!("Pattern matched: {}", pattern);
                    println!("Seed phrase: {}", mnemonic);
                    println!("Address: {}", address);
                    println!("Position: {}", position);
                    println!("---------------------------");
                })
                .take_any(num_results)
                .collect()
        };

        self.running.store(false, Ordering::Relaxed);
        progress_thread.join().unwrap();
        progress.finish_and_clear();

        matches
    }

    pub fn get_stats(&self) -> (usize, usize, f64, f64, usize) {
        let total_seeds = self.total_seeds.load(Ordering::Relaxed);
        let total_addresses = self.total_addresses.load(Ordering::Relaxed);
        let duration = self.start_time.elapsed().as_secs_f64();
        let seed_rate = total_seeds as f64 / duration;
        let address_rate = total_addresses as f64 / duration;
        (total_seeds, total_addresses, seed_rate, address_rate, self.thread_count)
    }
}