// Crypto acceleration module
//
// This module provides hardware-accelerated cryptographic operations
// when available on the current CPU.

/// CPU features detected at runtime
#[derive(Debug, Clone, Copy, Default)]
pub struct CpuFeatures {
    pub sse2: bool,
    pub sse4_1: bool, 
    pub avx: bool,
    pub avx2: bool,
    pub avx512f: bool,
}

impl CpuFeatures {
    /// Get optimal batch size for SIMD operations based on available CPU features
    pub fn optimal_batch_size(&self) -> usize {
        if self.avx512f {
            16  // Process 16 hashes in parallel with AVX-512
        } else if self.avx2 {
            8   // Process 8 hashes in parallel with AVX2 
        } else if self.avx {
            4   // Process 4 hashes in parallel with AVX
        } else if self.sse4_1 || self.sse2 {
            2   // Process 2 hashes in parallel with SSE2/SSE4.1
        } else {
            1   // No SIMD - process one at a time
        }
    }
    
    /// Returns a multiplier for batch sizes that's optimal for the current CPU
    pub fn batch_size_multiplier(&self) -> usize {
        let base = self.optimal_batch_size();
        base * 128  // Larger batches for better throughput
    }
}

/// Detect CPU features at runtime
pub fn detect_cpu_features() -> CpuFeatures {
    let mut features = CpuFeatures::default();
    
    #[cfg(target_arch = "x86_64")]
    {
        // Safe check for available CPU features
        features.sse2 = is_x86_feature_detected!("sse2");
        features.sse4_1 = is_x86_feature_detected!("sse4.1");
        features.avx = is_x86_feature_detected!("avx");
        features.avx2 = is_x86_feature_detected!("avx2");
        features.avx512f = is_x86_feature_detected!("avx512f");
    }
    
    features
}

/// Context for cryptographic acceleration
pub struct AccelContext {
    pub features: CpuFeatures,
    pub use_hw_accel: bool,
}

impl AccelContext {
    pub fn new() -> Self {
        let features = detect_cpu_features();
        // Use hardware acceleration if AVX or better is available
        let use_hw_accel = cfg!(feature = "hw_accel") && 
            (features.avx || features.avx2 || features.avx512f);
            
        Self {
            features,
            use_hw_accel,
        }
    }
    
    pub fn get_optimal_batch_size(&self) -> usize {
        if self.use_hw_accel {
            self.features.optimal_batch_size()
        } else {
            1 // No hardware acceleration
        }
    }
    
    pub fn get_optimal_batch_count(&self) -> usize {
        if self.use_hw_accel {
            self.features.batch_size_multiplier()
        } else {
            1000 // Default batch size
        }
    }
    
    /// Log detected features
    pub fn log_features(&self) {
        if self.use_hw_accel {
            let features = &self.features;
            let mut available = Vec::new();
            
            if features.sse2 { available.push("SSE2"); }
            if features.sse4_1 { available.push("SSE4.1"); }
            if features.avx { available.push("AVX"); }
            if features.avx2 { available.push("AVX2"); }
            if features.avx512f { available.push("AVX-512F"); }
            
            if !available.is_empty() {
                println!("Using hardware acceleration with: {}", available.join(", "));
                println!("Optimal batch size: {}", self.get_optimal_batch_size());
            }
        }
    }
}

// This singleton ensures we only detect CPU features once
lazy_static::lazy_static! {
    pub static ref ACCEL_CONTEXT: AccelContext = {
        AccelContext::new()
    };
}

/// Get the global acceleration context
pub fn get_context() -> &'static AccelContext {
    &ACCEL_CONTEXT
} 