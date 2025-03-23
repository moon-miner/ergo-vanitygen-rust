use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use indicatif::{ProgressBar, ProgressStyle};

pub type StatsSummary = (usize, usize, f64, f64, usize);
pub type ProgressCallback = Box<dyn Fn(usize, usize, f64, f64) + Send + Sync>;

/// Handles progress tracking, statistics, and callbacks
pub struct ProgressTracker {
    pub total_seeds: Arc<AtomicUsize>,
    pub total_addresses: Arc<AtomicUsize>,
    pub running: Arc<AtomicBool>,
    pub start_time: Instant,
    thread_count: usize,
    callback: Arc<Mutex<Option<ProgressCallback>>>,
    progress_bar: Option<Arc<ProgressBar>>,
    // EMA weight factors for smoothing
    smoothing_factor: f64,
    // Measurement interval for stability
    update_interval_secs: f64,
}

impl ProgressTracker {
    pub fn new(thread_count: usize, show_progress_bar: bool) -> Self {
        let progress_bar = if show_progress_bar {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} [{elapsed_precise}] {msg}")
                    .unwrap()
            );
            Some(Arc::new(pb))
        } else {
            None
        };

        ProgressTracker {
            total_seeds: Arc::new(AtomicUsize::new(0)),
            total_addresses: Arc::new(AtomicUsize::new(0)),
            running: Arc::new(AtomicBool::new(true)),
            start_time: Instant::now(),
            thread_count,
            callback: Arc::new(Mutex::new(None)),
            progress_bar,
            // Lower smoothing factor (20%) for more stable rates
            smoothing_factor: 0.2,
            // Slightly longer interval for more stable measurements
            update_interval_secs: 0.5,
        }
    }

    /// Set a callback function to receive progress updates
    pub fn set_callback<F>(&self, callback: F)
    where
        F: Fn(usize, usize, f64, f64) + Send + Sync + 'static,
    {
        *self.callback.lock().unwrap() = Some(Box::new(callback));
    }

    /// Record processed data
    pub fn record_processed(&self, seeds: usize, addresses: usize) {
        self.total_seeds.fetch_add(seeds, Ordering::Relaxed);
        self.total_addresses.fetch_add(addresses, Ordering::Relaxed);
    }

    /// Start progress monitoring thread
    pub fn start_monitoring_thread(&self) -> std::thread::JoinHandle<()> {
        // Set the running flag to true to start monitoring
        self.running.store(true, Ordering::SeqCst);
        
        let total_seeds = self.total_seeds.clone();
        let total_addresses = self.total_addresses.clone();
        let running = self.running.clone();
        let callback = self.callback.clone();
        let progress_bar = self.progress_bar.clone();
        let smoothing_factor = self.smoothing_factor;
        let update_interval = self.update_interval_secs;
        
        std::thread::spawn(move || {
            let mut last_seeds = 0;
            let mut last_addresses = 0;
            let mut last_time = Instant::now();
            let mut first_update = true;
            
            // Track rates with smoothing
            let mut smoothed_seed_rate = 0.0;
            let mut smoothed_addr_rate = 0.0;
            
            // History for median filtering
            let mut seed_rate_history: Vec<f64> = Vec::with_capacity(5);
            let mut addr_rate_history: Vec<f64> = Vec::with_capacity(5);

            while running.load(Ordering::Relaxed) {
                let current_seeds = total_seeds.load(Ordering::Relaxed);
                let current_addresses = total_addresses.load(Ordering::Relaxed);
                let current_time = Instant::now();
                
                let delta_seeds = current_seeds - last_seeds;
                let delta_addresses = current_addresses - last_addresses;
                let delta_time = current_time.duration_since(last_time).as_secs_f64();
                
                // Update at regular intervals
                if delta_time >= update_interval {
                    // Avoid division by zero or very small intervals
                    if delta_time > 0.001 {
                        // Calculate instantaneous rates
                        let instant_seed_rate = delta_seeds as f64 / delta_time;
                        let instant_addr_rate = delta_addresses as f64 / delta_time;
                        
                        // Add rates to history for median filtering
                        seed_rate_history.push(instant_seed_rate);
                        addr_rate_history.push(instant_addr_rate);
                        
                        // Keep history at a reasonable size
                        if seed_rate_history.len() > 5 {
                            seed_rate_history.remove(0);
                            addr_rate_history.remove(0);
                        }
                        
                        // Apply median filtering to reject outliers
                        let mut seed_rates_sorted = seed_rate_history.clone();
                        seed_rates_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                        
                        let mut addr_rates_sorted = addr_rate_history.clone();
                        addr_rates_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                        
                        // Use median value if we have enough history points
                        let filtered_seed_rate = if seed_rates_sorted.len() >= 3 {
                            seed_rates_sorted[seed_rates_sorted.len() / 2]
                        } else {
                            instant_seed_rate
                        };
                        
                        let filtered_addr_rate = if addr_rates_sorted.len() >= 3 {
                            addr_rates_sorted[addr_rates_sorted.len() / 2]
                        } else {
                            instant_addr_rate
                        };
                        
                        if first_update {
                            // On first update, just use the filtered rate directly
                            smoothed_seed_rate = filtered_seed_rate;
                            smoothed_addr_rate = filtered_addr_rate;
                            first_update = false;
                        } else {
                            // Apply exponential moving average for further smoothing
                            smoothed_seed_rate = smoothing_factor * filtered_seed_rate + 
                                              (1.0 - smoothing_factor) * smoothed_seed_rate;
                            smoothed_addr_rate = smoothing_factor * filtered_addr_rate + 
                                              (1.0 - smoothing_factor) * smoothed_addr_rate;
                        };
                    }
                    
                    // Update progress display
                    if let Some(pb) = &progress_bar {
                        pb.set_message(format!(
                            "Checked {} seeds ({:.0} seeds/s) and {} addresses ({:.0} addr/s)...",
                            current_seeds, smoothed_seed_rate, current_addresses, smoothed_addr_rate
                        ));
                    }
                    
                    // Call progress callback if set
                    if let Some(ref cb) = *callback.lock().unwrap() {
                        cb(current_seeds, current_addresses, smoothed_seed_rate, smoothed_addr_rate);
                    }
                    
                    last_seeds = current_seeds;
                    last_addresses = current_addresses;
                    last_time = current_time;
                }

                // Shorter sleep time for more responsive updates
                std::thread::sleep(Duration::from_millis(50));
            }

            // Clear progress bar when done
            if let Some(pb) = &progress_bar {
                pb.finish_and_clear();
            }
        })
    }

    /// Stop progress tracking
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    /// Get final statistics
    pub fn get_stats(&self) -> StatsSummary {
        let total_seeds = self.total_seeds.load(Ordering::Relaxed);
        let total_addresses = self.total_addresses.load(Ordering::Relaxed);
        let duration = self.start_time.elapsed().as_secs_f64();
        
        // Avoid division by zero
        let seed_rate = if duration > 0.0 { total_seeds as f64 / duration } else { 0.0 };
        let address_rate = if duration > 0.0 { total_addresses as f64 / duration } else { 0.0 };
        
        (total_seeds, total_addresses, seed_rate, address_rate, self.thread_count)
    }

    /// Resets the progress tracker to its initial state
    pub fn reset(&self) {
        // Make sure the monitoring thread is stopped
        self.running.store(false, Ordering::SeqCst);
        
        // Reset counters
        self.total_seeds.store(0, Ordering::Relaxed);
        self.total_addresses.store(0, Ordering::Relaxed);
        
        // Reset progress bar if present
        if let Some(pb) = &self.progress_bar {
            pb.reset();
        }
        
        // We'll set running back to true when start_monitoring_thread is called
        // Don't set it here, as that would create a race condition
    }
} 