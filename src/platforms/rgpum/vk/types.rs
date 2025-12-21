//! High-level types for RGPUM

use super::ffi;

// Error type
#[derive(Debug, Clone, Copy)]
pub struct VkError(pub ffi::VkResult);

impl std::fmt::Display for VkError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0 {
            ffi::VK_SUCCESS => write!(f, "Success"),
            ffi::VK_ERROR_INITIALIZATION_FAILED => write!(f, "Initialization failed"),
            code => write!(f, "Vulkan error code: {}", code),
        }
    }
}

impl std::error::Error for VkError {}

/// Physical device information
#[derive(Debug, Clone)]
pub struct PhysicalDeviceInfo {
    pub name: String,
    pub device_type: DeviceType,
    pub api_version: (u32, u32, u32),
    pub vendor_id: u32,
    pub device_id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    DiscreteGpu = 2,
    IntegratedGpu = 1,
    VirtualGpu = 3,
    Cpu = 4,
    Other = 0,
}

impl DeviceType {
    pub fn from_u32(value: u32) -> Self {
        match value {
            1 => DeviceType::IntegratedGpu,
            2 => DeviceType::DiscreteGpu,
            3 => DeviceType::VirtualGpu,
            4 => DeviceType::Cpu,
            _ => DeviceType::Other,
        }
    }
}

/// Queue family information
#[derive(Debug, Clone)]
pub struct QueueFamilyInfo {
    pub queue_flags: u32,
    pub queue_count: u32,
    pub supports_graphics: bool,
}
