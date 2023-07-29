use ash::vk;

use crate::renderer::core::Core;
use crate::renderer::device::Device;
use crate::renderer::image::Image;

pub struct Swapchain {
    pub swapchain_init: ash::extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,

    pub image_count: u32,
    pub images: Vec<Image>,
}

impl Swapchain {
    pub unsafe fn new(c: &Core, d: &Device) -> Swapchain {
        assert!(d.surface_capabilities.max_image_count >= 2, "Swapchain doesn't support 2 images");

        let image_count = if d.surface_capabilities.max_image_count > 0 && d.surface_capabilities.min_image_count + 1 > d.surface_capabilities.max_image_count {
            d.surface_capabilities.max_image_count
        } else {
            d.surface_capabilities.min_image_count + 1
        };

        //let image_count = 2;

        let (queue_family_indices, sharing_mode) = if d.queue_present.1 == d.queue_main.1 {
            (vec![d.queue_present.1], vk::SharingMode::EXCLUSIVE)
        } else {
            (vec![d.queue_present.1, d.queue_main.1], vk::SharingMode::CONCURRENT)
        };

        let present_mode = d.surface_init.get_physical_device_surface_present_modes(d.physical_device, d.surface).unwrap().iter().cloned().find(|&pm| pm == vk::PresentModeKHR::MAILBOX).unwrap_or(vk::PresentModeKHR::FIFO);
        //let present_mode = vk::PresentModeKHR::IMMEDIATE;

        let swapchain_init = ash::extensions::khr::Swapchain::new(&c.instance, &d.device);

        let swapchain_ci = vk::SwapchainCreateInfoKHR::builder()
            .surface(d.surface)
            .min_image_count(image_count)
            .image_format(d.surface_format.format)
            .image_color_space(d.surface_format.color_space)
            .image_extent(d.surface_extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(sharing_mode)
            .queue_family_indices(&queue_family_indices)
            .pre_transform(d.surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        let swapchain = swapchain_init.create_swapchain(&swapchain_ci, None).unwrap();

        let image_handles = swapchain_init.get_swapchain_images(swapchain).unwrap();
        
        let images: Vec<Image> = image_handles.iter().map(|&i| {
            let image_view_ci = vk::ImageViewCreateInfo::builder()
                .image(i)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(d.surface_format.format)
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::R,
                    g: vk::ComponentSwizzle::G,
                    b: vk::ComponentSwizzle::B,
                    a: vk::ComponentSwizzle::A,
               })
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });

            let image_view = d.device.create_image_view(&image_view_ci, None).unwrap();
            
            let extent = vk::Extent3D {
                width: d.surface_extent.width,
                height: d.surface_extent.height,
                depth: 1,
            };

            Image {
                image: i,
                view: image_view,
                memory: None,
                width: d.surface_extent.width,
                height: d.surface_extent.height,
                extent,
            }
        }).collect();

        Swapchain {
            swapchain_init,
            swapchain,

            image_count,
            images,
        }
    }
}