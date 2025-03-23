use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use rayon::prelude::*;
use crate::utils::{generate_mnemonic, generate_addresses};
use crate::progress::{ProgressTracker, StatsSummary};
use crate::matcher::PatternMatcher;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;

// Result type containing mnemonic, address, matched pattern, address position, and seed word count
pub type MatchResult = (String, String, String, u32, usize);

/// Address processor for finding vanity addresses
pub struct AddressProcessor {
    progress: ProgressTracker,
    max_batch_size: usize,
    min_batch_size: usize,
    batch_adjust_interval: usize,
    batch_size: Arc<AtomicUsize>,
    batch_counter: Arc<AtomicUsize>,
    performance_metrics: Arc<Mutex<HashMap<usize, Duration>>>,
    should_cancel: Arc<AtomicBool>,
    result_callback: Arc<Mutex<Option<Box<dyn Fn(&str, &str, &str, u32, usize) + Send + Sync>>>>,
}

impl AddressProcessor {
    pub fn new() -> Self {
        let cpu_count = num_cpus::get();
        let thread_count = cpu_count.max(1);
        
        // Configure thread pool only once
        // Use a static flag to track if we've already set up the thread pool
        static THREAD_POOL_INITIALIZED: AtomicBool = AtomicBool::new(false);
        
        if !THREAD_POOL_INITIALIZED.load(Ordering::SeqCst) {
            // Only try to configure the thread pool if we haven't done so already
            if let Err(e) = rayon::ThreadPoolBuilder::new()
                .num_threads(thread_count)
                .build_global() 
            {
                eprintln!("Warning: Failed to configure thread pool: {}", e);
            } else {
                // Successfully initialized
                THREAD_POOL_INITIALIZED.store(true, Ordering::SeqCst);
            }
        }

        // Dynamic batch sizing parameters
        let initial_batch_size = 1000;
        let min_batch_size = 100;
        let max_batch_size = 5000;
        let batch_adjust_interval = 10;

        Self {
            progress: ProgressTracker::new(thread_count, true),
            max_batch_size,
            min_batch_size,
            batch_adjust_interval,
            batch_size: Arc::new(AtomicUsize::new(initial_batch_size)),
            batch_counter: Arc::new(AtomicUsize::new(0)),
            performance_metrics: Arc::new(Mutex::new(HashMap::new())),
            should_cancel: Arc::new(AtomicBool::new(false)),
            result_callback: Arc::new(Mutex::new(None)),
        }
    }

    /// Set a callback for progress updates
    pub fn set_progress_callback<F>(&self, callback: F)
    where
        F: Fn(usize, usize, f64, f64) + Send + Sync + 'static,
    {
        // Wrap the callback to throttle updates
        let throttled_callback = move |seeds, addresses, seed_rate, addr_rate| {
            // Only call back every 250ms for performance
            static LAST_UPDATE: AtomicUsize = AtomicUsize::new(0);
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as usize;
            
            if now - LAST_UPDATE.load(Ordering::Relaxed) > 250 {
                LAST_UPDATE.store(now, Ordering::Relaxed);
                callback(seeds, addresses, seed_rate, addr_rate);
            }
        };
        
        self.progress.set_callback(throttled_callback);
    }

    /// Find addresses matching the given patterns
    pub fn find_matches(
        &self,
        matcher: PatternMatcher,
        word_count: usize,
        num_results: usize,
        balanced: bool,
        addresses_per_seed: u32,
    ) -> Vec<MatchResult> {
        // Adjust initial batch size based on word count
        let initial_batch_size = if word_count == 0 {
            // For mixed word counts, use a moderate batch size
            1000
        } else if word_count == 12 {
            1200 // 20% higher for 12-word seed phrases
        } else {
            1000
        };
        self.batch_size.store(initial_batch_size, Ordering::Relaxed);
        
        // Start progress monitoring thread
        let progress_thread = self.progress.start_monitoring_thread();

        // Find matches using appropriate strategy
        let matches = if balanced {
            self.find_balanced_matches(&matcher, word_count, num_results, addresses_per_seed)
        } else {
            self.find_any_matches(&matcher, word_count, num_results, addresses_per_seed)
        };

        // Stop progress tracking and wait for thread to finish
        self.progress.stop();
        let _ = progress_thread.join();

        matches
    }

    /// Get performance statistics
    pub fn get_stats(&self) -> StatsSummary {
        self.progress.get_stats()
    }

    /// Request cancellation of the current search
    pub fn cancel(&self) {
        // Set the cancellation flag with the strongest memory ordering
        self.should_cancel.store(true, Ordering::SeqCst);
        
        // Stop progress tracking immediately
        self.progress.stop();
        
        // Log the cancellation request
        println!("Cancellation requested - stopping search");
    }
    
    /// Reset the processor for a new search
    pub fn reset(&self) {
        // Reset cancellation flag
        self.should_cancel.store(false, Ordering::SeqCst);
        
        // Reset batch counter
        self.batch_counter.store(0, Ordering::Relaxed);
        
        // Clear performance metrics
        self.performance_metrics.lock().unwrap().clear();
        
        // Reset progress tracker
        self.progress.reset();
    }
    
    /// Check if the search has been cancelled
    fn is_cancelled(&self) -> bool {
        self.should_cancel.load(Ordering::SeqCst)
    }

    // Find matches balanced across all patterns
    fn find_balanced_matches(
        &self,
        matcher: &PatternMatcher,
        word_count: usize,
        num_results: usize,
        addresses_per_seed: u32,
    ) -> Vec<MatchResult> {
        let pattern_count = matcher.pattern_count();
        let _per_pattern = (num_results + pattern_count - 1) / pattern_count;
        
        let pattern_matches = Arc::new(Mutex::new(HashMap::<String, usize>::new()));
        let found_count = Arc::new(AtomicUsize::new(0));
        let results = Arc::new(Mutex::new(Vec::new()));
        
        let should_cancel = self.should_cancel.clone();
        
        rayon::iter::repeat(())
            .map_init(
                || {},
                |_, _| {
                    // Check for cancellation and if we've found enough results
                    if should_cancel.load(Ordering::SeqCst) || 
                       found_count.load(Ordering::SeqCst) >= num_results {
                        return Vec::new();
                    }
                    
                    // Track performance for this batch
                    let thread_id = rayon::current_thread_index().unwrap_or(0);
                    let batch_num = self.batch_counter.fetch_add(1, Ordering::Relaxed);
                    let current_batch_size = if cfg!(target_os = "linux") {
                        // For Linux/AppImage, use larger batches for fewer callbacks
                        self.batch_size.load(Ordering::Relaxed).max(2000)
                    } else {
                        self.batch_size.load(Ordering::Relaxed)
                    };
                    
                    // Adjust batch size periodically based on thread performance
                    if batch_num % self.batch_adjust_interval == 0 {
                        self.adjust_batch_size(thread_id);
                    }
                    
                    let start_time = Instant::now();
                    let mut batch_results = Vec::new();
                    
                    for _ in 0..current_batch_size {
                        // Check for cancellation periodically and if we've found enough results
                        if should_cancel.load(Ordering::SeqCst) || 
                           found_count.load(Ordering::SeqCst) >= num_results {
                            break;
                        }
                        
                        let (mnemonic, actual_word_count) = generate_mnemonic(word_count);
                        let addresses = generate_addresses(&mnemonic, addresses_per_seed);
                        
                        for addr_info in addresses {
                            if let Some(pattern) = matcher.is_match(&addr_info.address) {
                                let mut map = pattern_matches.lock().unwrap();
                                let count = map.entry(pattern.clone()).or_insert(0);
                                *count += 1;
                                batch_results.push((
                                    mnemonic.clone(),
                                    addr_info.address,
                                    pattern.clone(),
                                    addr_info.position,
                                    actual_word_count
                                ));
                            }
                        }
                    }
                    
                    // Record performance for this batch
                    let elapsed = start_time.elapsed();
                    self.performance_metrics.lock().unwrap().insert(thread_id, elapsed);
                    
                    // Record statistics
                    self.progress.record_processed(current_batch_size, 
                                                  current_batch_size * addresses_per_seed as usize);
                    
                    batch_results
                }
            )
            .flatten()
            .filter(|_| !should_cancel.load(Ordering::SeqCst))
            .inspect(|(mnemonic, address, pattern, position, actual_word_count)| {
                // Skip if cancelled
                if should_cancel.load(Ordering::SeqCst) {
                    return;
                }
                
                let mut map = pattern_matches.lock().unwrap();
                let current_count = map.entry(pattern.clone()).or_insert(0);
                *current_count += 1;
                let total_found = found_count.fetch_add(1, Ordering::Relaxed) + 1;
                
                // Store the result for immediate access in real-time
                results.lock().unwrap().push((
                    mnemonic.clone(),
                    address.clone(),
                    pattern.clone(),
                    *position,
                    *actual_word_count
                ));
                
                // Call the callback if set, in addition to printing to standard output
                if let Some(callback) = self.result_callback.lock().unwrap().as_ref() {
                    callback(mnemonic, address, pattern, *position, *actual_word_count);
                }
                
                // Display result immediately for CLI mode
                println!("\n---------------------------");
                println!("Match {} of {}", total_found, num_results);
                println!("Pattern matched: {}", pattern);
                println!("Seed phrase ({}-word): {}", actual_word_count, mnemonic);
                println!("Address: {}", address);
                println!("Position: {}", position);
                println!("---------------------------");
            })
            .take_any(num_results)
            .collect::<Vec<_>>();
            
        // Fix the "does not live long enough" error by storing the result in a variable
        let return_results = results.lock().unwrap().clone();
        return_results
    }

    // Find any matches up to the requested number
    fn find_any_matches(
        &self,
        matcher: &PatternMatcher,
        word_count: usize,
        num_results: usize,
        addresses_per_seed: u32,
    ) -> Vec<MatchResult> {
        let found_count = Arc::new(AtomicUsize::new(0));
        let results = Arc::new(Mutex::new(Vec::new()));
        
        let should_cancel = self.should_cancel.clone();
        
        rayon::iter::repeat(())
            .map_init(
                || {},
                |_, _| {
                    // Check for cancellation and if we've found enough results
                    if should_cancel.load(Ordering::SeqCst) || 
                       found_count.load(Ordering::SeqCst) >= num_results {
                        return Vec::new();
                    }
                    
                    // Track performance for this batch
                    let thread_id = rayon::current_thread_index().unwrap_or(0);
                    let batch_num = self.batch_counter.fetch_add(1, Ordering::Relaxed);
                    let current_batch_size = self.batch_size.load(Ordering::Relaxed);
                    
                    // Adjust batch size periodically based on thread performance
                    if batch_num % self.batch_adjust_interval == 0 {
                        self.adjust_batch_size(thread_id);
                    }
                    
                    let start_time = Instant::now();
                    let mut batch_results = Vec::new();
                    
                    for _ in 0..current_batch_size {
                        // Check for cancellation periodically and if we've found enough results
                        if should_cancel.load(Ordering::SeqCst) || 
                           found_count.load(Ordering::SeqCst) >= num_results {
                            break;
                        }
                        
                        let (mnemonic, actual_word_count) = generate_mnemonic(word_count);
                        let addresses = generate_addresses(&mnemonic, addresses_per_seed);
                        
                        for addr_info in addresses {
                            if let Some(pattern) = matcher.is_match(&addr_info.address) {
                                batch_results.push((
                                    mnemonic.clone(),
                                    addr_info.address,
                                    pattern,
                                    addr_info.position,
                                    actual_word_count
                                ));
                            }
                        }
                    }
                    
                    // Record performance for this batch
                    let elapsed = start_time.elapsed();
                    self.performance_metrics.lock().unwrap().insert(thread_id, elapsed);
                    
                    // Record statistics
                    self.progress.record_processed(current_batch_size, 
                                                  current_batch_size * addresses_per_seed as usize);
                    
                    batch_results
                }
            )
            .flatten()
            .filter(|_| !should_cancel.load(Ordering::SeqCst))
            .inspect(|(mnemonic, address, pattern, position, actual_word_count)| {
                // Skip if cancelled
                if should_cancel.load(Ordering::SeqCst) {
                    return;
                }
                
                let total_found = found_count.fetch_add(1, Ordering::Relaxed) + 1;
                
                // Store the result for immediate access in real-time
                results.lock().unwrap().push((
                    mnemonic.clone(),
                    address.clone(),
                    pattern.clone(),
                    *position,
                    *actual_word_count
                ));
                
                // Call the callback if set, in addition to printing to standard output
                if let Some(callback) = self.result_callback.lock().unwrap().as_ref() {
                    callback(mnemonic, address, pattern, *position, *actual_word_count);
                }
                
                // Display result immediately for CLI mode
                println!("\n---------------------------");
                println!("Match {} of {}", total_found, num_results);
                println!("Pattern matched: {}", pattern);
                println!("Seed phrase ({}-word): {}", actual_word_count, mnemonic);
                println!("Address: {}", address);
                println!("Position: {}", position);
                println!("---------------------------");
            })
            .take_any(num_results)
            .collect::<Vec<_>>();
            
        // Fix the "does not live long enough" error by storing the result in a variable
        let return_results = results.lock().unwrap().clone();
        return_results
    }
    
    // Adjust batch size based on thread performance
    fn adjust_batch_size(&self, thread_id: usize) {
        let metrics_lock = self.performance_metrics.lock().unwrap();
        
        // If we don't have enough data yet, don't adjust
        if metrics_lock.len() < 3 {
            return;
        }
        
        let current_time = metrics_lock.get(&thread_id).cloned().unwrap_or(Duration::from_millis(1000));
        
        // Find the median processing time
        let mut times: Vec<Duration> = metrics_lock.values().cloned().collect();
        times.sort();
        let median_time = times[times.len() / 2];
        
        // Get current batch size
        let current_batch_size = self.batch_size.load(Ordering::Relaxed);
        
        // More gradual adjustments - 10% instead of 20%
        let adjustment_factor = 0.1;
        
        // Only make adjustments if there's a significant difference (>50% rather than >100%)
        if current_time > median_time.mul_f64(1.5) && current_batch_size > self.min_batch_size {
            // Reduce batch size more gradually
            let new_size = ((current_batch_size as f64) * (1.0 - adjustment_factor)).max(self.min_batch_size as f64) as usize;
            self.batch_size.store(new_size, Ordering::Relaxed);
        } 
        // If this thread is significantly faster than median, increase its batch size
        else if current_time.mul_f64(1.5) < median_time && current_batch_size < self.max_batch_size {
            // Increase batch size more gradually
            let new_size = ((current_batch_size as f64) * (1.0 + adjustment_factor)).min(self.max_batch_size as f64) as usize;
            self.batch_size.store(new_size, Ordering::Relaxed);
        }
    }

    // Add a method to set the result callback
    pub fn set_result_callback<F>(&self, callback: F)
    where
        F: Fn(&str, &str, &str, u32, usize) + Send + Sync + 'static,
    {
        *self.result_callback.lock().unwrap() = Some(Box::new(callback));
    }
}