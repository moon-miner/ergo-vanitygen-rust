use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use rayon::prelude::*;
use indicatif::{ProgressBar, ProgressStyle};
use crate::utils::{generate_mnemonic, generate_address};
use std::sync::Mutex;

pub struct AddressProcessor {
    total_checked: Arc<AtomicUsize>,
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
            total_checked: Arc::new(AtomicUsize::new(0)),
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
    ) -> Vec<(String, String, String)> {
        let progress = Arc::new(ProgressBar::new_spinner());
        progress.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} [{elapsed_precise}] {msg}")
                .unwrap()
        );

        let total_checked = self.total_checked.clone();
        let running = self.running.clone();
        let progress_clone = progress.clone();
        
        let progress_thread = std::thread::spawn(move || {
            let mut last_checked = 0;
            let mut last_time = Instant::now();
            let mut max_rate = 0f64;
            let mut first_update = true;

            while running.load(Ordering::Relaxed) {
                let current_checked = total_checked.load(Ordering::Relaxed);
                let current_time = Instant::now();
                
                let delta_checked = current_checked - last_checked;
                let delta_time = current_time.duration_since(last_time).as_secs_f64();
                
                if delta_time >= 0.1 {
                    let rate = delta_checked as f64 / delta_time;
                    
                    if !first_update {
                        max_rate = max_rate.max(rate);
                    }
                    first_update = false;

                    progress_clone.set_message(format!(
                        "Checked {} addresses at {:.0} addr/s (max: {:.0} addr/s)...",
                        current_checked, rate, max_rate
                    ));

                    last_checked = current_checked;
                    last_time = current_time;
                }

                std::thread::sleep(Duration::from_secs(1));
            }
        });

        let matches: Vec<(String, String, String)> = if balanced {
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
                            self.total_checked.fetch_add(1, Ordering::Relaxed);
                            let mnemonic = generate_mnemonic(word_count);
                            let address = generate_address(&mnemonic);
                            if let Some(pattern) = matcher(&address) {
                                let count = pattern_matches_clone.lock().unwrap()
                                    .get(&pattern).copied().unwrap_or(0);
                                if count < per_pattern {
                                    batch_results.push((mnemonic, address, pattern.clone()));
                                }
                            }
                        }
                        batch_results
                    }
                )
                .flatten()
                .inspect(|(mnemonic, address, pattern)| {
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
                            self.total_checked.fetch_add(1, Ordering::Relaxed);
                            let mnemonic = generate_mnemonic(word_count);
                            let address = generate_address(&mnemonic);
                            if let Some(pattern) = matcher(&address) {
                                batch_results.push((mnemonic, address, pattern));
                            }
                        }
                        batch_results
                    }
                )
                .flatten()
                .inspect(|(mnemonic, address, pattern)| {
                    let total_found = found_count_clone.fetch_add(1, Ordering::Relaxed) + 1;
                    
                    // Display result immediately
                    println!("\n---------------------------");
                    println!("Match {} of {}", total_found, num_results);
                    println!("Pattern matched: {}", pattern);
                    println!("Seed phrase: {}", mnemonic);
                    println!("Address: {}", address);
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

    pub fn get_stats(&self) -> (usize, f64, usize) {
        let total = self.total_checked.load(Ordering::Relaxed);
        let duration = self.start_time.elapsed().as_secs_f64();
        let rate = total as f64 / duration;
        (total, rate, self.thread_count)
    }
}