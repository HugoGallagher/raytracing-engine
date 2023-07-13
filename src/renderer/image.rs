use ash::vk;

use crate::renderer::core::Core;
use crate::renderer::device::Device;
use crate::renderer::sampler::Sampler;

#[derive(Copy, Clone)]
pub struct Image2D {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub memory: Option<vk::DeviceMemory>,
    pub width: u32,
    pub height: u32,
    pub extent: vk::Extent2D,
}

#[derive(Copy, Clone)]
pub struct Image2DBuilder {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub usage: Option<vk::ImageUsageFlags>,
    pub format: Option<vk::Format>,
}

impl Image2DBuilder {
    pub fn new() -> Image2DBuilder {
        Image2DBuilder {
            width: None,
            height: None,
            usage: None,
            format: None,
        }
    }
    pub fn width(&self, width: u32) -> Image2DBuilder {
        Image2DBuilder {
            width: Some(width),
            height: self.height,
            usage: self.usage,
            format: self.format,
        }
    }
    
    pub fn height(&self, height: u32) -> Image2DBuilder {
        Image2DBuilder {
            width: self.width,
            height: Some(height),
            usage: self.usage,
            format: self.format,
        }
    }
    
    pub fn usage(&self, usage: vk::ImageUsageFlags) -> Image2DBuilder {
        Image2DBuilder {
            width: self.width,
            height: self.height,
            usage: Some(usage),
            format: self.format,
        }
    }
    
    pub fn format(&self, format: vk::Format) -> Image2DBuilder {
        Image2DBuilder {
            width: self.width,
            height: self.height,
            usage: self.usage,
            format: Some(format),
        }
    }

    pub unsafe fn build(&self, c: &Core, d: &Device) -> Image2D {
        Image2D::new(
            c, d,
            self.width.expect("Error: Image builder has no specified width"),
            self.height.expect("Error: Image builder has no specified height"),
            self.usage.expect("Error: Image builder has no specified usage"),
            self.format.expect("Error: Image builder has no specified format"),
        )
    }

    pub unsafe fn build_many(&self, c: &Core, d: &Device, count: u32) -> Vec<Image2D> {
        let mut images = Vec::<Image2D>::new();
        for _ in 0..count {
            images.push(Image2D::new(
                c, d,
                self.width.expect("Error: Image builder has no specified width"),
                self.height.expect("Error: Image builder has no specified height"),
                self.usage.expect("Error: Image builder has no specified usage"),
                self.format.expect("Error: Image builder has no specified format"),
            ));
        }

        images
    }
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

    pub unsafe fn generate_samplers(c: &Core, d: &Device, images: &Vec<Image2D>) -> Vec<Sampler> {
        let mut samplers = Vec::<Sampler>::new();
        for image in images {
            samplers.push(Sampler::new(c, d, image.view))
        }

        samplers
    }
}