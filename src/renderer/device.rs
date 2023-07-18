use ash::vk;
use raw_window_handle::{RawWindowHandle, RawDisplayHandle};

use crate::renderer::core::Core;

pub struct Device {
    pub device: ash::Device,

    pub surface_init: ash::extensions::khr::Surface,
    pub surface: vk::SurfaceKHR,
    pub surface_format: vk::SurfaceFormatKHR,
    pub surface_capabilities: vk::SurfaceCapabilitiesKHR,
    pub surface_extent: vk::Extent2D,

    pub extension_names: Vec<*const i8>,

    pub physical_device: vk::PhysicalDevice,

    pub queue_present: (vk::Queue, u32),
    pub queue_graphics: (vk::Queue, u32),
    pub queue_compute: (vk::Queue, u32),
}

impl Device {
    pub unsafe fn new(c: &Core, window: RawWindowHandle, display: RawDisplayHandle) -> Device {
        let surface_init = ash::extensions::khr::Surface::new(&c.entry, &c.instance);
        let surface = ash_window::create_surface(&c.entry, &c.instance, display, window, None).unwrap();

        let available_physical_devices = c.instance.enumerate_physical_devices().unwrap();

        let (physical_device, queue_index_present, queue_index_graphics, queue_index_compute) = available_physical_devices.iter().filter_map(|&pd| {
            let queue_family_properties = c.instance.get_physical_device_queue_family_properties(pd);

            let queue_index_properties_present = queue_family_properties.iter().enumerate().filter(|(i, ref q)| {
                surface_init.get_physical_device_surface_support(pd, *i as u32, surface).unwrap()
            }).next();
            let queue_index_properties_graphics = queue_family_properties.iter().enumerate().filter(|(i, ref q)| {
                q.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            }).next();
            let queue_index_properties_compute = queue_family_properties.iter().enumerate().filter(|(i, ref q)| {
                q.queue_flags.contains(vk::QueueFlags::COMPUTE)
            }).next();

            if queue_index_properties_present.is_some() && queue_index_properties_graphics.is_some() && queue_index_properties_compute.is_some() {
                Some((pd, queue_index_properties_present.unwrap().0 as u32, queue_index_properties_graphics.unwrap().0 as u32, queue_index_properties_compute.unwrap().0 as u32))
            } else {
                None
            }
        }).next().expect("Suitable physical device not found");

        let extension_names = vec![ash::extensions::khr::Swapchain::name().as_ptr()];

        let physical_device_features = vk::PhysicalDeviceFeatures {
            shader_clip_distance: 1,
            ..Default::default()
        };

        let priorities = [1.0];

        let queue_indices = vec![queue_index_present, queue_index_graphics, queue_index_compute];
        let mut unique_indices = Vec::<u32>::new();
        let mut queue_cis = Vec::<vk::DeviceQueueCreateInfo>::new();

        queue_indices.iter().for_each(|&i| {
            if !unique_indices.contains(&i) {
                unique_indices.push(i);
            }
        });

        unique_indices.iter().for_each(|&i| {
            let queue_ci = vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(i)
                .queue_priorities(&priorities)
                .build();

            queue_cis.push(queue_ci);
        });

        let device_ci = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_cis)
            .enabled_extension_names(&extension_names)
            .enabled_features(&physical_device_features);

        let device = c.instance.create_device(physical_device, &device_ci, None).unwrap();

        let queue_present = (device.get_device_queue(queue_index_present, 0), queue_index_present);
        let queue_graphics = (device.get_device_queue(queue_index_graphics, 0), queue_index_graphics);
        let queue_compute = (device.get_device_queue(queue_index_compute, 0), queue_index_compute);

        let available_surface_formats = surface_init.get_physical_device_surface_formats(physical_device, surface).unwrap();
        let surface_format = available_surface_formats.iter().filter(|format| {
            format.format == vk::Format::B8G8R8A8_SRGB && format.color_space == vk::ColorSpaceKHR::EXTENDED_SRGB_NONLINEAR_EXT
        }).next().unwrap_or(&available_surface_formats[0]);

        let surface_capabilities = surface_init.get_physical_device_surface_capabilities(physical_device, surface).unwrap();

        let surface_extent = if surface_capabilities.current_extent.width == std::u32::MAX {
            vk::Extent2D {
                width: 1280,
                height: 720,
            }
        } else {
            surface_capabilities.current_extent
        };

        Device {
            device: device,

            surface_init: surface_init,
            surface: surface,
            surface_format: *surface_format,
            surface_capabilities: surface_capabilities,
            surface_extent: surface_extent,

            extension_names: extension_names,

            physical_device: physical_device,

            queue_present: queue_present,
            queue_graphics: queue_graphics,
            queue_compute: queue_compute,
        }
    }

    pub unsafe fn get_memory_type(&self, c: &Core, property_flags: vk::MemoryPropertyFlags, memory_requirements: vk::MemoryRequirements) -> usize {
        let memory_type_index = c.instance.get_physical_device_memory_properties(self.physical_device).memory_types.iter().enumerate().find_map(|(i, m)| {
            if (memory_requirements.memory_type_bits & (1 << i)) != 0 && (m.property_flags & property_flags == property_flags) {
                Some(i)
            } else {
                None
            }
        }).unwrap();

        memory_type_index
    }
}