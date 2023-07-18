use ash::vk;

use crate::renderer::core::Core;
use crate::renderer::device::Device;
use crate::renderer::sampler::Sampler;

#[derive(Copy, Clone)]
pub struct Image {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub memory: Option<vk::DeviceMemory>,
    pub width: u32,
    pub height: u32,
    pub extent: vk::Extent3D,
}

#[derive(Copy, Clone)]
pub struct ImageBuilder {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub depth: Option<u32>,
    pub usage: Option<vk::ImageUsageFlags>,
    pub format: Option<vk::Format>,
}

impl ImageBuilder {
    pub fn new() -> ImageBuilder {
        ImageBuilder {
            width: None,
            height: None,
            depth: None,
            usage: None,
            format: None,
        }
    }
    pub fn width(&self, width: u32) -> ImageBuilder {
        ImageBuilder {
            width: Some(width),
            height: self.height,
            depth: self.depth,
            usage: self.usage,
            format: self.format,
        }
    }
    
    pub fn height(&self, height: u32) -> ImageBuilder {
        ImageBuilder {
            width: self.width,
            height: Some(height),
            depth: self.depth,
            usage: self.usage,
            format: self.format,
        }
    }
    
    pub fn depth(&self, depth: u32) -> ImageBuilder {
        ImageBuilder {
            width: self.width,
            height: self.height,
            depth: Some(depth),
            usage: self.usage,
            format: self.format,
        }
    }
    
    pub fn usage(&self, usage: vk::ImageUsageFlags) -> ImageBuilder {
        ImageBuilder {
            width: self.width,
            height: self.height,
            depth: self.depth,
            usage: Some(usage),
            format: self.format,
        }
    }
    
    pub fn format(&self, format: vk::Format) -> ImageBuilder {
        ImageBuilder {
            width: self.width,
            height: self.height,
            depth: self.depth,
            usage: self.usage,
            format: Some(format),
        }
    }

    pub unsafe fn build(&self, c: &Core, d: &Device) -> Image {
        Image::new(
            c, d,
            self.width.expect("Error: Image builder has no specified width"),
            self.height.expect("Error: Image builder has no specified height"),
            self.depth,
            self.usage.expect("Error: Image builder has no specified usage"),
            self.format.expect("Error: Image builder has no specified format"),
        )
    }

    pub unsafe fn build_many(&self, c: &Core, d: &Device, count: usize) -> Vec<Image> {
        let mut images = Vec::<Image>::new();
        for _ in 0..count {
            images.push(Image::new(
                c, d,
                self.width.expect("Error: Image builder has no specified width"),
                self.height.expect("Error: Image builder has no specified height"),
                self.depth,
                self.usage.expect("Error: Image builder has no specified usage"),
                self.format.expect("Error: Image builder has no specified format"),
            ));
        }

        images
    }
}

impl Image {
    pub unsafe fn new(c: &Core, d: &Device, w: u32, h: u32, de: Option<u32>, u: vk::ImageUsageFlags, format: vk::Format) -> Image {
        let (image_type, depth, extent_depth) = match de {
            Some(dep) => (vk::ImageType::TYPE_3D, Some(dep), dep),
            None => (vk::ImageType::TYPE_2D, None, 1),
        };
        
        let extent = vk::Extent3D::builder()
            .width(w)
            .height(h)
            .depth(extent_depth)
            .build();

        let image_ci = vk::ImageCreateInfo::builder()
            .image_type(image_type)
            .extent(extent)
            .mip_levels(1)
            .array_layers(1)
            .format(format)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(u)
            .samples(vk::SampleCountFlags::TYPE_1);

        let image = d.device.create_image(&image_ci, None).unwrap();

        let memory_requirements = d.device.get_image_memory_requirements(image);
        let memory_type_index = d.get_memory_type(c, vk::MemoryPropertyFlags::DEVICE_LOCAL, memory_requirements);

        let memory_alloc_i = vk::MemoryAllocateInfo::builder()
            .allocation_size(memory_requirements.size)
            .memory_type_index(memory_type_index as u32);

        let memory = d.device.allocate_memory(&memory_alloc_i, None).unwrap();
        d.device.bind_image_memory(image, memory, 0).unwrap();

        let image_aspect = match u {
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT => vk::ImageAspectFlags::DEPTH,
            _ => vk::ImageAspectFlags::COLOR,
        };

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
                aspect_mask: image_aspect,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        let view = d.device.create_image_view(&view_ci, None).unwrap();

        Image {
            image,
            view,
            memory: Some(memory),
            width: w,
            height: h,
            extent,
        }
    }

    pub unsafe fn generate_samplers(c: &Core, d: &Device, images: &Vec<Image>) -> Vec<Sampler> {
        let mut samplers = Vec::<Sampler>::new();
        for image in images {
            samplers.push(Sampler::new(c, d, image.view))
        }

        samplers
    }
}