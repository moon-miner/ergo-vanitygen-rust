use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use rayon::prelude::*;
use indicatif::{ProgressBar, ProgressStyle};
use crate::utils::{generate_mnemonic, generate_address};

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
        matcher: impl Fn(&str) -> bool + Send + Sync,
        word_count: usize
    ) -> Vec<(String, String)> {
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
                
                // Only calculate rate if we have a meaningful time delta
                if delta_time >= 0.1 { // At least 100ms
                    let rate = delta_checked as f64 / delta_time;
                    
                    // Skip the first update to avoid initial spike
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

        // Simple batch processing
        let matches: Vec<(String, String)> = rayon::iter::repeat(())
            .map_init(
                || (),
                |_, _| {
                    (0..self.batch_size)
                        .into_par_iter()
                        .map(|_| {
                            self.total_checked.fetch_add(1, Ordering::Relaxed);
                            let mnemonic = generate_mnemonic(word_count);
                            let address = generate_address(&mnemonic);
                            (mnemonic, address)
                        })
                        .find_first(|(_, addr)| matcher(addr))
                }
            )
            .find_first(|result| result.is_some())
            .and_then(|x| x)
            .into_iter()
            .collect();

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