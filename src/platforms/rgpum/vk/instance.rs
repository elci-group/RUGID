//! Vulkan instance with real dynamic loading

use super::ffi;
use super::types::*;
use std::ffi::{CStr, CString};
use std::ptr;

pub struct Instance {
    handle: ffi::VkInstance,
    _lib: libloading::Library,
    
    // Function pointers
    destroy_instance: ffi::PFN_vkDestroyInstance,
    enumerate_physical_devices: ffi::PFN_vkEnumeratePhysicalDevices,
    get_physical_device_properties: ffi::PFN_vkGetPhysicalDeviceProperties,
    get_queue_family_properties: ffi::PFN_vkGetPhysicalDeviceQueueFamilyProperties,
}

impl Instance {
    /// Create a new Vulkan instance with real dynamic loading
    pub fn new(
        app_name: &str,
        enable_validation: bool,
    ) -> Result<Self, VkError> {
        unsafe {
            log::info!("🔧 Loading Vulkan library...");
            
            // Load Vulkan shared library
            let lib_name = if cfg!(target_os = "windows") {
                "vulkan-1.dll"
            } else if cfg!(target_os = "macos") {
                "libMoltenVK.dylib"
            } else {
                "libvulkan.so.1"
            };
            
            let lib = libloading::Library::new(lib_name)
                .map_err(|e| {
                    log::error!("Failed to load {}: {}", lib_name, e);
                    VkError(ffi::VK_ERROR_INITIALIZATION_FAILED)
                })?;
            
            log::info!("✅ Vulkan library loaded: {}", lib_name);
            
            // Get vkGetInstanceProcAddr
            let get_proc_addr: libloading::Symbol<ffi::PFN_vkGetInstanceProcAddr> = lib
                .get(b"vkGetInstanceProcAddr")
                .map_err(|_| VkError(ffi::VK_ERROR_INITIALIZATION_FAILED))?;
            
            // Load vkCreateInstance
            let create_instance_name = CString::new("vkCreateInstance").unwrap();
            let create_instance: ffi::PFN_vkCreateInstance = std::mem::transmute(
                get_proc_addr(ptr::null_mut(), create_instance_name.as_ptr())
            );
            
            // Prepare application info
            let app_name_c = CString::new(app_name).unwrap();
            let engine_name = CString::new("RGPUM").unwrap();
            
            let app_info = ffi::VkApplicationInfo {
                sType: ffi::VK_STRUCTURE_TYPE_APPLICATION_INFO,
                pNext: ptr::null(),
                pApplicationName: app_name_c.as_ptr(),
                applicationVersion: ffi::vk_make_version(1, 0, 0),
                pEngineName: engine_name.as_ptr(),
                engineVersion: ffi::vk_make_version(1, 0, 0),
                apiVersion: ffi::VK_API_VERSION_1_2,
            };
            
            // Extensions
            #[cfg(target_os = "linux")]
            let extensions = vec![
                b"VK_KHR_surface\0".as_ptr() as *const i8,
                b"VK_KHR_xlib_surface\0".as_ptr() as *const i8,
                b"VK_KHR_xcb_surface\0".as_ptr() as *const i8,
                b"VK_KHR_wayland_surface\0".as_ptr() as *const i8,
            ];
            
            #[cfg(target_os = "windows")]
            let extensions = vec![
                b"VK_KHR_surface\0".as_ptr() as *const i8,
                b"VK_KHR_win32_surface\0".as_ptr() as *const i8,
            ];
            
            #[cfg(target_os = "macos")]
            let extensions = vec![
                b"VK_KHR_surface\0".as_ptr() as *const i8,
                b"VK_EXT_metal_surface\0".as_ptr() as *const i8,
            ];
            
            // Validation layers
            let layers: Vec<*const i8> = if enable_validation {
                log::info!("🛡️  Enabling validation layers");
                vec![b"VK_LAYER_KHRONOS_validation\0".as_ptr() as *const i8]
            } else {
                vec![]
            };
            
            let create_info = ffi::VkInstanceCreateInfo {
                sType: ffi::VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
                pNext: ptr::null(),
                flags: 0,
                pApplicationInfo: &app_info,
                enabledLayerCount: layers.len() as u32,
                ppEnabledLayerNames: layers.as_ptr(),
                enabledExtensionCount: extensions.len() as u32,
                ppEnabledExtensionNames: extensions.as_ptr(),
            };
            
            // Create instance!
            let mut handle = ptr::null_mut();
            let result = create_instance(&create_info, ptr::null(), &mut handle);
            
            if result != ffi::VK_SUCCESS {
                log::error!("vkCreateInstance failed: {}", result);
                return Err(VkError(result));
            }
            
            log::info!("✅ Vulkan instance created");
            
            // Load instance-level functions
            let destroy_name = CString::new("vkDestroyInstance").unwrap();
            let destroy_instance = std::mem::transmute(
                get_proc_addr(handle, destroy_name.as_ptr())
            );
            
            let enum_name = CString::new("vkEnumeratePhysicalDevices").unwrap();
            let enumerate_physical_devices = std::mem::transmute(
                get_proc_addr(handle, enum_name.as_ptr())
            );
            
            let props_name = CString::new("vkGetPhysicalDeviceProperties").unwrap();
            let get_physical_device_properties = std::mem::transmute(
                get_proc_addr(handle, props_name.as_ptr())
            );
            
            let queue_name = CString::new("vkGetPhysicalDeviceQueueFamilyProperties").unwrap();
            let get_queue_family_properties = std::mem::transmute(
                get_proc_addr(handle, queue_name.as_ptr())
            );
            
            Ok(Self {
                handle,
                _lib: lib,
                destroy_instance,
                enumerate_physical_devices,
                get_physical_device_properties,
                get_queue_family_properties,
            })
        }
    }
    
    /// Enumerate physical devices (GPUs)
    pub fn enumerate_devices(&self) -> Result<Vec<ffi::VkPhysicalDevice>, VkError> {
        unsafe {
            let mut count = 0;
            let result = (self.enumerate_physical_devices)(
                self.handle,
                &mut count,
                ptr::null_mut(),
            );
            
            if result != ffi::VK_SUCCESS {
                return Err(VkError(result));
            }
            
            if count == 0 {
                log::warn!("No Vulkan devices found!");
                return Ok(vec![]);
            }
            
            let mut devices = vec![ptr::null_mut(); count as usize];
            let result = (self.enumerate_physical_devices)(
                self.handle,
                &mut count,
                devices.as_mut_ptr(),
            );
            
            if result != ffi::VK_SUCCESS {
                return Err(VkError(result));
            }
            
            Ok(devices)
        }
    }
    
    /// Get device properties
    pub fn get_device_info(&self, device: ffi::VkPhysicalDevice) -> PhysicalDeviceInfo {
        unsafe {
            let mut props: ffi::VkPhysicalDeviceProperties = std::mem::zeroed();
            (self.get_physical_device_properties)(device, &mut props);
            
            // Extract device name
            let name_bytes: &[u8] = std::slice::from_raw_parts(
                props.deviceName.as_ptr() as *const u8,
                256,
            );
            let name_cstr = CStr::from_bytes_until_nul(name_bytes)
                .unwrap_or(CStr::from_bytes_with_nul(b"Unknown\0").unwrap());
            let name = name_cstr.to_string_lossy().to_string();
            
            // Parse API version
            let api_major = props.apiVersion >> 22;
            let api_minor = (props.apiVersion >> 12) & 0x3FF;
            let api_patch = props.apiVersion & 0xFFF;
            
            PhysicalDeviceInfo {
                name,
                device_type: DeviceType::from_u32(props.deviceType),
                api_version: (api_major, api_minor, api_patch),
                vendor_id: props.vendorID,
                device_id: props.deviceID,
            }
        }
    }
    
    /// Get queue family properties
    pub fn get_queue_families(&self, device: ffi::VkPhysicalDevice) -> Vec<QueueFamilyInfo> {
        unsafe {
            let mut count = 0;
            (self.get_queue_family_properties)(device, &mut count, ptr::null_mut());
            
            if count == 0 {
                return vec![];
            }
            
            let mut props: Vec<ffi::VkQueueFamilyProperties> = Vec::with_capacity(count as usize);
            props.set_len(count as usize);
            (self.get_queue_family_properties)(device, &mut count, props.as_mut_ptr());
            
            props.into_iter().map(|p| QueueFamilyInfo {
                queue_flags: p.queueFlags,
                queue_count: p.queueCount,
                supports_graphics: (p.queueFlags & ffi::VK_QUEUE_GRAPHICS_BIT) != 0,
            }).collect()
        }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            log::info!("🧹 Destroying Vulkan instance");
            (self.destroy_instance)(self.handle, ptr::null());
        }
    }
}
