//! RGPUM - RUGID Graphics Processor Unit Manager
//! 
//! High-performance custom graphics backend with direct Vulkan API access.
//! Designed to replace wgpu with optimized buffer management and reduced overhead.

mod vk;  // Custom minimal Vulkan wrapper
mod simple_platform;

// Old implementations moved to .old files for future reference
// mod buffer_pool;
// mod pipeline_cache;
// mod command_pool;

pub use simple_platform::RgpumPlatform;

use crate::platform::{Platform, PlatformEvent};

/// RGPUM configuration
pub struct RgpumConfig {
    /// Number of frames in flight (triple buffering recommended)
    pub frames_in_flight: usize,
    /// Enable validation layers (debug builds)
    pub enable_validation: bool,
    /// Initial buffer pool size (bytes)
    pub initial_buffer_size: usize,
}

impl Default for RgpumConfig {
    fn default() -> Self {
        Self {
            frames_in_flight: 3,
            enable_validation: cfg!(debug_assertions),
            initial_buffer_size: 4 * 1024 * 1024, // 4MB
        }
    }
}
