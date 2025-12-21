//! Raw Vulkan FFI types and constants
//!
//! Minimal definitions needed for RGPUM - we load functions dynamically

use std::os::raw::{c_char, c_void};

// Vulkan handles (opaque pointers)
pub type VkInstance = *mut c_void;
pub type VkPhysicalDevice = *mut c_void;
pub type VkDevice = *mut c_void;

// Result codes
pub type VkResult = i32;
pub const VK_SUCCESS: VkResult = 0;
pub const VK_ERROR_INITIALIZATION_FAILED: VkResult = -3;

// API version macros
pub const fn vk_make_version(major: u32, minor: u32, patch: u32) -> u32 {
    (major << 22) | (minor << 12) | patch
}

pub const VK_API_VERSION_1_2: u32 = vk_make_version(1, 2, 0);

// Structure types
pub const VK_STRUCTURE_TYPE_APPLICATION_INFO: u32 = 0;
pub const VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO: u32 = 1;

// Application info
#[repr(C)]
pub struct VkApplicationInfo {
    pub sType: u32,
    pub pNext: *const c_void,
    pub pApplicationName: *const c_char,
    pub applicationVersion: u32,
    pub pEngineName: *const c_char,
    pub engineVersion: u32,
    pub apiVersion: u32,
}

// Instance create info
#[repr(C)]
pub struct VkInstanceCreateInfo {
    pub sType: u32,
    pub pNext: *const c_void,
    pub flags: u32,
    pub pApplicationInfo: *const VkApplicationInfo,
    pub enabledLayerCount: u32,
    pub ppEnabledLayerNames: *const *const c_char,
    pub enabledExtensionCount: u32,
    pub ppEnabledExtensionNames: *const *const c_char,
}

// Physical device properties
#[repr(C)]
pub struct VkPhysicalDeviceProperties {
    pub apiVersion: u32,
    pub driverVersion: u32,
    pub vendorID: u32,
    pub deviceID: u32,
    pub deviceType: u32,
    pub deviceName: [c_char; 256],
    // ... other fields we don't need yet
}

// Queue family properties
#[repr(C)]
pub struct VkQueueFamilyProperties {
    pub queueFlags: u32,
    pub queueCount: u32,
    pub timestampValidBits: u32,
    pub minImageTransferGranularity: VkExtent3D,
}

#[repr(C)]
pub struct VkExtent3D {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

// Queue flags
pub const VK_QUEUE_GRAPHICS_BIT: u32 = 0x00000001;

// Function pointer types
pub type PFN_vkVoidFunction = *const c_void;

pub type PFN_vkGetInstanceProcAddr = unsafe extern "system" fn(
    instance: VkInstance,
    pName: *const c_char,
) -> PFN_vkVoidFunction;

pub type PFN_vkCreateInstance = unsafe extern "system" fn(
    pCreateInfo: *const VkInstanceCreateInfo,
    pAllocator: *const c_void,
    pInstance: *mut VkInstance,
) -> VkResult;

pub type PFN_vkDestroyInstance = unsafe extern "system" fn(
    instance: VkInstance,
    pAllocator: *const c_void,
);

pub type PFN_vkEnumeratePhysicalDevices = unsafe extern "system" fn(
    instance: VkInstance,
    pPhysicalDeviceCount: *mut u32,
    pPhysicalDevices: *mut VkPhysicalDevice,
) -> VkResult;

pub type PFN_vkGetPhysicalDeviceProperties = unsafe extern "system" fn(
    physicalDevice: VkPhysicalDevice,
    pProperties: *mut VkPhysicalDeviceProperties,
);

pub type PFN_vkGetPhysicalDeviceQueueFamilyProperties = unsafe extern "system" fn(
    physicalDevice: VkPhysicalDevice,
    pQueueFamilyPropertyCount: *mut u32,
    pQueueFamilyProperties: *mut VkQueueFamilyProperties,
);
