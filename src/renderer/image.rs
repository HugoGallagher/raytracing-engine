use ash::version::{InstanceV1_0, DeviceV1_0};
use ash::vk;

use crate::renderer::core::Core;
use crate::renderer::device::Device;

#[derive(Copy, Clone)]
pub struct Image2D {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub memory: Option<vk::DeviceMemory>,
    pub width: u32,
    pub height: u32,
    pub extent: vk::Extent2D,
}

pub struct Image2DBuilder {
    pub width: u32,
    pub height: u32,
    pub usage: vk::ImageUsageFlags,
    pub format: vk::Format,
}

impl Image2D {
    pub unsafe fn new(c: &Core, d: &Device, w: u32, h: u32, u: vk::ImageUsageFlags, format: vk::Format) -> Image2D {
        let extent_3d = vk::Extent3D::builder()
            .width(w)
            .height(h)
            .depth(1)
            .build();

        let extent = vk::Extent2D {
            width: w,
            height: h,
        };

        let image_ci = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(extent_3d)
            .mip_levels(1)
            .array_layers(1)
            .format(format)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(u)
            .samples(vk::SampleCountFlags::TYPE_1);

        let image = d.device.create_image(&image_ci, None).unwrap();

        let memory_requirements = d.device.get_image_memory_requirements(image);
        let memory_type_index = c.instance.get_physical_device_memory_properties(d.physical_device).memory_types.iter().enumerate().find_map(|(i, m)| {
            if (memory_requirements.memory_type_bits & (1 << i)) != 0 && (m.property_flags & vk::MemoryPropertyFlags::DEVICE_LOCAL) == vk::MemoryPropertyFlags::DEVICE_LOCAL {
                Some(i)
            } else {
                None
            }
        }).unwrap();

        let memory_alloc_i = vk::MemoryAllocateInfo::builder()
            .allocation_size(memory_requirements.size)
            .memory_type_index(memory_type_index as u32);

        let memory = d.device.allocate_memory(&memory_alloc_i, None).unwrap();
        d.device.bind_image_memory(image, memory, 0).unwrap();

        let view_ci = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
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

        let view = d.device.create_image_view(&view_ci, None).unwrap();

        Image2D {
            image: image,
            view: view,
            memory: Some(memory),
            width: w,
            height: h,
            extent: extent,
        }
    }
}