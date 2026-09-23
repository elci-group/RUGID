//! Custom Vulkan wrapper for RGPUM

#![allow(dead_code)]

pub mod commands;
pub mod device; // Device stubs
pub mod ffi; // Raw FFI definitions
pub mod instance; // Instance management
pub mod memory;
pub mod swapchain;
pub mod sync;
pub mod types; // High-level types

pub use instance::Instance;
pub use types::*;

/// Result type for Vulkan operations
pub type VkResult<T> = Result<T, VkError>;
