//! Vulkan backend implementation for RGPUM
//! 
//! Direct Vulkan API access with optimized buffer management and minimal overhead.

use ash::{vk, Entry};
use ash::extensions::khr;
use std::sync::Arc;
use std::ffi::{CStr, CString};
use winit::window::Window;

use crate::platform::{Platform, PlatformEvent};
use crate::input::InputEvent;
use super::buffer_pool::BufferPool;
use super::command_pool::CommandPool;
use super::pipeline_cache::PipelineCache;
use super::RgpumConfig;

/// RGPUM platform with Vulkan backend
pub struct RgpumPlatform {
    // Core Vulkan objects
    _entry: Entry,
    instance: ash::Instance,
    surface: vk::SurfaceKHR,
    surface_loader: khr::Surface,
    physical_device: vk::PhysicalDevice,
    device: Arc<ash::Device>,
    queue: vk::Queue,
    queue_family_index: u32,
    
    // Swapchain
    swapchain: vk::SwapchainKHR,
    swapchain_loader: khr::Swapchain,
    swapchain_images: Vec<vk::Image>,
    swapchain_image_views: Vec<vk::ImageView>,
    swapchain_extent: vk::Extent2D,
    swapchain_format: vk::Format,
    
    // Rendering resources
    render_pass: vk::RenderPass,
    framebuffers: Vec<vk::Framebuffer>,
    
    // RGPUM optimizations
    buffer_pool: BufferPool,
    command_pool: CommandPool,
    pipeline_cache: PipelineCache,
    
    // Synchronization
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    current_frame: usize,
    
    // Event queue
    events: Vec<PlatformEvent>,
    
    // Config
    config: RgpumConfig,
}

impl RgpumPlatform {
    pub fn new(window: Arc<Window>, config: RgpumConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let entry = unsafe { Entry::load()? };
        
        // Create instance
        let app_name = CString::new("RUGID")?;
        let engine_name = CString::new("RGPUM")?;
        let app_info = vk::ApplicationInfo {
            s_type: vk::StructureType::APPLICATION_INFO,
            p_next: std::ptr::null(),
            p_application_name: app_name.as_ptr(),
            application_version: vk::make_api_version(0, 1, 0, 0),
            p_engine_name: engine_name.as_ptr(),
            engine_version: vk::make_api_version(0, 1, 0, 0),
            api_version: vk::API_VERSION_1_2,
        };
        
        // Extensions
        let mut extension_names = vec![
            khr::Surface::name().as_ptr(),
        ];
        
        // Platform-specific extensions
        #[cfg(target_os = "linux")]
        {
            extension_names.push(khr::XlibSurface::name().as_ptr());
            extension_names.push(khr::WaylandSurface::name().as_ptr());
        }
        #[cfg(target_os = "macos")]
        {
            extension_names.push(ash::extensions::ext::MetalSurface::name().as_ptr());
        }
        #[cfg(target_os = "windows")]
        {
            extension_names.push(khr::Win32Surface::name().as_ptr());
        }
        
        // Validation layers (debug only)
        let layer_names = if config.enable_validation {
            vec![CString::new("VK_LAYER_KHRONOS_validation")?]
        } else {
            vec![]
        };
        let layer_name_ptrs: Vec<*const i8> = layer_names.iter()
            .map(|name| name.as_ptr())
            .collect();
        
        let create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::InstanceCreateFlags::empty(),
            p_application_info: &app_info,
            enabled_layer_count: layer_name_ptrs.len() as u32,
            pp_enabled_layer_names: layer_name_ptrs.as_ptr(),
            enabled_extension_count: extension_names.len() as u32,
            pp_enabled_extension_names: extension_names.as_ptr(),
        };
        
        let instance = unsafe { entry.create_instance(&create_info, None)? };
        
        // Create surface
        let surface = unsafe { Self::create_surface(&entry, &instance, &window)? };
        let surface_loader = khr::Surface::new(&entry, &instance);
        
        // Pick physical device
        let physical_device = Self::pick_physical_device(&instance, &surface_loader, surface)?;
        
        // Get queue family
        let queue_family_index = Self::find_queue_family(&instance, physical_device, &surface_loader, surface)?;
        
        // Create logical device
        let device_extension_names = vec![khr::Swapchain::name().as_ptr()];
        let queue_priorities = [1.0];
        let queue_create_info = vk::DeviceQueueCreateInfo {
            s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::DeviceQueueCreateFlags::empty(),
            queue_family_index,
            queue_count: queue_priorities.len() as u32,
            p_queue_priorities: queue_priorities.as_ptr(),
        };
        
        let queue_create_infos = [queue_create_info];
        let device_create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::DeviceCreateFlags::empty(),
            queue_create_info_count: queue_create_infos.len() as u32,
            p_queue_create_infos: queue_create_infos.as_ptr(),
            enabled_layer_count: 0,
            pp_enabled_layer_names: std::ptr::null(),
            enabled_extension_count: device_extension_names.len() as u32,
            pp_enabled_extension_names: device_extension_names.as_ptr(),
            p_enabled_features: std::ptr::null(),
        };
        
        let device = Arc::new(unsafe { instance.create_device(physical_device, &device_create_info, None)? });
        let queue = unsafe { device.get_device_queue(queue_family_index, 0) };
        
        // Create swapchain
        let surface_capabilities = unsafe {
            surface_loader.get_physical_device_surface_capabilities(physical_device, surface)?
        };
        let surface_format = Self::choose_surface_format(&surface_loader, physical_device, surface)?;
        let present_mode = Self::choose_present_mode(&surface_loader, physical_device, surface)?;
        let extent = Self::choose_extent(&surface_capabilities, &window);
        
        let image_count = (config.frames_in_flight as u32).max(surface_capabilities.min_image_count);
        
        let swapchain_create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            p_next: std::ptr::null(),
            flags: vk::SwapchainCreateFlagsKHR::empty(),
            surface,
            min_image_count: image_count,
            image_format: surface_format.format,
            image_color_space: surface_format.color_space,
            image_extent: extent,
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: std::ptr::null(),
            pre_transform: surface_capabilities.current_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode,
            clipped: vk::TRUE,
            old_swapchain: vk::SwapchainKHR::null(),
        };
        
        let swapchain_loader = khr::Swapchain::new(&instance, &device);
        let swapchain = unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None)? };
        let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };
        
        // Create image views
        let swapchain_image_views: Result<Vec<_>, _> = swapchain_images.iter().map(|&image| {
            let create_info = vk::ImageViewCreateInfo {
                s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                p_next: std::ptr::null(),
                flags: vk::ImageViewCreateFlags::empty(),
                image,
                view_type: vk::ImageViewType::TYPE_2D,
                format: surface_format.format,
                components: vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                },
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
            };
            
            unsafe { device.create_image_view(&create_info, None) }
        }).collect();
        let swapchain_image_views = swapchain_image_views?;
        
        // Create render pass
        let color_attachment = vk::AttachmentDescription {
            flags: vk::AttachmentDescriptionFlags::empty(),
            format: surface_format.format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
        };
        
        let color_attachment_ref = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };
        
        let color_attachments = [color_attachment_ref];
        let subpass = vk::SubpassDescription {
            flags: vk::SubpassDescriptionFlags::empty(),
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            input_attachment_count: 0,
            p_input_attachments: std::ptr::null(),
            color_attachment_count: color_attachments.len() as u32,
            p_color_attachments: color_attachments.as_ptr(),
            p_resolve_attachments: std::ptr::null(),
            p_depth_stencil_attachment: std::ptr::null(),
            preserve_attachment_count: 0,
            p_preserve_attachments: std::ptr::null(),
        };
        
        let dependency = vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask: vk::AccessFlags::empty(),
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dependency_flags: vk::DependencyFlags::empty(),
        };
        
        let attachments = [color_attachment];
        let subpasses = [subpass];
        let dependencies = [dependency];
        let render_pass_info = vk::RenderPassCreateInfo {
            s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::RenderPassCreateFlags::empty(),
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            subpass_count: subpasses.len() as u32,
            p_subpasses: subpasses.as_ptr(),
            dependency_count: dependencies.len() as u32,
            p_dependencies: dependencies.as_ptr(),
        };
        
        let render_pass = unsafe { device.create_render_pass(&render_pass_info, None)? };
        
        // Create framebuffers
        let framebuffers: Result<Vec<_>, _> = swapchain_image_views.iter().map(|&view| {
            let attachments = [view];
            let framebuffer_info = vk::FramebufferCreateInfo {
                s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
                p_next: std::ptr::null(),
                flags: vk::FramebufferCreateFlags::empty(),
                render_pass,
                attachment_count: attachments.len() as u32,
                p_attachments: attachments.as_ptr(),
                width: extent.width,
                height: extent.height,
                layers: 1,
            };
            
            unsafe { device.create_framebuffer(&framebuffer_info, None) }
        }).collect();
        let framebuffers = framebuffers?;
        
        // Create synchronization objects
        let semaphore_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::SemaphoreCreateFlags::empty(),
        };
        let mut image_available_semaphores = Vec::new();
        let mut render_finished_semaphores = Vec::new();
        
        for _ in 0..config.frames_in_flight {
            image_available_semaphores.push(unsafe { device.create_semaphore(&semaphore_info, None)? });
            render_finished_semaphores.push(unsafe { device.create_semaphore(&semaphore_info, None)? });
        }
        
        // Create RGPUM components
        let memory_properties = unsafe { instance.get_physical_device_memory_properties(physical_device) };
        let buffer_pool = BufferPool::new(
            Arc::clone(&device),
            &memory_properties,
            config.initial_buffer_size as vk::DeviceSize,
            config.frames_in_flight,
        )?;
        
        let command_pool = CommandPool::new(Arc::clone(&device), queue_family_index)?;
        let pipeline_cache = PipelineCache::new(Arc::clone(&device))?;
        
        log::info!("RGPUM initialized: Vulkan backend with {} frames in flight", config.frames_in_flight);
        
        Ok(Self {
            _entry: entry,
            instance,
            surface,
            surface_loader,
            physical_device,
            device,
            queue,
            queue_family_index,
            swapchain,
            swapchain_loader,
            swapchain_images,
            swapchain_image_views,
            swapchain_extent: extent,
            swapchain_format: surface_format.format,
            render_pass,
            framebuffers,
            buffer_pool,
            command_pool,
            pipeline_cache,
            image_available_semaphores,
            render_finished_semaphores,
            current_frame: 0,
            events: Vec::new(),
            config,
        })
    }
    
    // Helper functions
    
    unsafe fn create_surface(
        entry: &Entry,
        instance: &ash::Instance,
        window: &Window,
    ) -> Result<vk::SurfaceKHR, vk::Result> {
        use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
        
        let display_handle = window.display_handle().unwrap().as_raw();
        let window_handle = window.window_handle().unwrap().as_raw();
        
        unsafe {
            ash_window::create_surface(entry, instance, display_handle, window_handle, None)
        }
    }
    
    fn pick_physical_device(
        instance: &ash::Instance,
        surface_loader: &khr::Surface,
        surface: vk::SurfaceKHR,
    ) -> Result<vk::PhysicalDevice, Box<dyn std::error::Error>> {
        let devices = unsafe { instance.enumerate_physical_devices()? };
        
        for device in devices {
            if Self::is_device_suitable(instance, device, surface_loader, surface)? {
                let props = unsafe { instance.get_physical_device_properties(device) };
                let device_name = unsafe { CStr::from_ptr(props.device_name.as_ptr()) };
                log::info!("Selected GPU: {:?}", device_name);
                return Ok(device);
            }
        }
        
        Err("No suitable GPU found".into())
    }
    
    fn is_device_suitable(
        instance: &ash::Instance,
        device: vk::PhysicalDevice,
        surface_loader: &khr::Surface,
        surface: vk::SurfaceKHR,
    ) -> Result<bool, vk::Result> {
        // Check queue family support
        let has_graphics_queue = Self::find_queue_family(instance, device, surface_loader, surface).is_ok();
        
        // Check swapchain support
        let formats = unsafe { surface_loader.get_physical_device_surface_formats(device, surface)? };
        let present_modes = unsafe { surface_loader.get_physical_device_surface_present_modes(device, surface)? };
        
        Ok(has_graphics_queue && !formats.is_empty() && !present_modes.is_empty())
    }
    
    fn find_queue_family(
        instance: &ash::Instance,
        device: vk::PhysicalDevice,
        surface_loader: &khr::Surface,
        surface: vk::SurfaceKHR,
    ) -> Result<u32, Box<dyn std::error::Error>> {
        let queue_families = unsafe { instance.get_physical_device_queue_family_properties(device) };
        
        for (index, family) in queue_families.iter().enumerate() {
            if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                let present_support = unsafe {
                    surface_loader.get_physical_device_surface_support(device, index as u32, surface)?
                };
                
                if present_support {
                    return Ok(index as u32);
                }
            }
        }
        
        Err("No suitable queue family found".into())
    }
    
    fn choose_surface_format(
        surface_loader: &khr::Surface,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
    ) -> Result<vk::SurfaceFormatKHR, vk::Result> {
        let formats = unsafe { surface_loader.get_physical_device_surface_formats(physical_device, surface)? };
        
        // Prefer SRGB
        for format in &formats {
            if format.format == vk::Format::B8G8R8A8_SRGB && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR {
                return Ok(*format);
            }
        }
        
        Ok(formats[0])
    }
    
    fn choose_present_mode(
        surface_loader: &khr::Surface,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
    ) -> Result<vk::PresentModeKHR, vk::Result> {
        let modes = unsafe { surface_loader.get_physical_device_surface_present_modes(physical_device, surface)? };
        
        // Prefer MAILBOX for lower latency
        if modes.contains(&vk::PresentModeKHR::MAILBOX) {
            Ok(vk::PresentModeKHR::MAILBOX)
        } else {
            Ok(vk::PresentModeKHR::FIFO) // Guaranteed to be available
        }
    }
    
    fn choose_extent(capabilities: &vk::SurfaceCapabilitiesKHR, window: &Window) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            let size = window.inner_size();
            vk::Extent2D {
                width: size.width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: size.height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        }
    }
}

impl Platform for RgpumPlatform {
    fn poll_events(&mut self) -> Vec<PlatformEvent> {
        let events = self.events.clone();
        self.events.clear();
        events
    }
    
    fn handle_event(&mut self, _event: &winit::event::Event<()>) {
        // TODO: Implement event handling
    }
    
    fn render(&mut self, _svg: String) {
        // Acquire next image
        let image_index = unsafe {
            match self.swapchain_loader.acquire_next_image(
                self.swapchain,
                u64::MAX,
                self.image_available_semaphores[self.current_frame],
                vk::Fence::null(),
            ) {
                Ok((index, _)) => index,
                Err(_) => return, // Handle swapchain recreation
            }
        };
        
        // Get buffer from pool
        let (buffer, _ptr, fence) = self.buffer_pool.get_buffer();
        
        // Acquire command buffer
        let cmd = self.command_pool.acquire().unwrap();
        
        // Begin command buffer
        let begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: std::ptr::null(),
            flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            p_inheritance_info: std::ptr::null(),
        };
        unsafe { self.device.begin_command_buffer(cmd, &begin_info).unwrap() };
        
        // Begin render pass
        let clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.1, 0.1, 0.1, 1.0],
            },
        };
        
        let clear_values = [clear_value];
        let render_pass_info = vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            p_next: std::ptr::null(),
            render_pass: self.render_pass,
            framebuffer: self.framebuffers[image_index as usize],
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain_extent,
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
        };
        
        unsafe {
            self.device.cmd_begin_render_pass(cmd, &render_pass_info, vk::SubpassContents::INLINE);
            
            // TODO: Record rendering commands
            
            self.device.cmd_end_render_pass(cmd);
            self.device.end_command_buffer(cmd).unwrap();
        }
        
        // Submit
        let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];
        let command_buffers = [cmd];
        
        let submit_info = vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: std::ptr::null(),
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: command_buffers.len() as u32,
            p_command_buffers: command_buffers.as_ptr(),
            signal_semaphore_count: signal_semaphores.len() as u32,
            p_signal_semaphores: signal_semaphores.as_ptr(),
        };
        
        unsafe {
            self.device.queue_submit(self.queue, &[*submit_info], fence).unwrap();
        }
        
        // Present
        let swapchains = [self.swapchain];
        let image_indices = [image_index];
        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            p_next: std::ptr::null(),
            wait_semaphore_count: signal_semaphores.len() as u32,
            p_wait_semaphores: signal_semaphores.as_ptr(),
            swapchain_count: swapchains.len() as u32,
            p_swapchains: swapchains.as_ptr(),
            p_image_indices: image_indices.as_ptr(),
            p_results: std::ptr::null_mut(),
        };
        
        unsafe {
            let _ = self.swapchain_loader.queue_present(self.queue, &present_info);
        }
        
        // Advance frame
        self.buffer_pool.advance_frame();
        self.current_frame = (self.current_frame + 1) % self.config.frames_in_flight;
        self.command_pool.release(cmd);
    }
    
    fn dimensions(&self) -> (f32, f32) {
        (self.swapchain_extent.width as f32, self.swapchain_extent.height as f32)
    }
    
    fn set_title(&mut self, _title: &str) {
        // Window title is set externally via winit
    }
}

impl Drop for RgpumPlatform {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();
            
            // Cleanup synchronization
            for semaphore in &self.image_available_semaphores {
                self.device.destroy_semaphore(*semaphore, None);
            }
            for semaphore in &self.render_finished_semaphores {
                self.device.destroy_semaphore(*semaphore, None);
            }
            
            // Cleanup framebuffers
            for framebuffer in &self.framebuffers {
                self.device.destroy_framebuffer(*framebuffer, None);
            }
            
            // Cleanup render pass
            self.device.destroy_render_pass(self.render_pass, None);
            
            // Cleanup swapchain
            for view in &self.swapchain_image_views {
                self.device.destroy_image_view(*view, None);
            }
            self.swapchain_loader.destroy_swapchain(self.swapchain, None);
            
            // Note: buffer_pool, command_pool, pipeline_cache drop automatically
            
            // Cleanup device and instance
            self.surface_loader.destroy_surface(self.surface, None);
            // device drops automatically (Arc)
            self.instance.destroy_instance(None);
        }
    }
}
