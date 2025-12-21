//! Custom Vulkan wrapper for RGPUM

#![allow(dead_code)]

pub mod ffi;      // Raw FFI definitions
pub mod types;    // High-level types
pub mod instance; // Instance management
pub mod device;   // Device stubs
pub mod swapchain;
pub mod commands;
pub mod sync;
pub mod memory;

pub use instance::Instance;
pub use types::*;

/// Result type for Vulkan operations
pub type VkResult<T> = Result<T, VkError>;
