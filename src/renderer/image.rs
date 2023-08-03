use ash::vk;

use crate::renderer::core::Core;
use crate::renderer::device::Device;
use crate::renderer::sampler::Sampler;

#[derive(Clone)]
pub struct ImageBuilder {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub depth: Option<u32>,
    pub usage: Option<vk::ImageUsageFlags>,
    pub format: Option<vk::Format>,
    pub pre_allocated_images: Option<Vec<vk::Image>>,
}


#[derive(Copy, Clone)]
pub struct Image {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub memory: Option<vk::DeviceMemory>,
    pub width: u32,
    pub height: u32,
    pub extent: vk::Extent3D,
}
impl ImageBuilder {
    pub fn new() -> ImageBuilder {
        ImageBuilder {
            width: None,
            height: None,
            depth: None,
            usage: None,
            format: None,
            pre_allocated_images: None,
        }
    }

    pub fn width(mut self, width: u32) -> ImageBuilder {
        self.width = Some(width);

        self
    }
    
    pub fn height(mut self, height: u32) -> ImageBuilder {
        self.height = Some(height);

        self
    }
    
    pub fn depth(mut self, depth: u32) -> ImageBuilder {
        self.depth = Some(depth);

        self
    }
    
    pub fn usage(mut self, usage: vk::ImageUsageFlags) -> ImageBuilder {
        self.usage = Some(usage);

        self
    }
    
    pub fn format(mut self, format: vk::Format) -> ImageBuilder {
        self.format = Some(format);

        self
    }
    
    pub fn pre_allocated_images(mut self, pre_allocated_images: Vec<vk::Image>) -> ImageBuilder {
        self.pre_allocated_images = Some(pre_allocated_images);

        self
    }

    pub unsafe fn build(&self, c: &Core, d: &Device) -> Image {
        let mut pre_allocated_image: Option<vk::Image> = None;
        if let Some(is) = self.pre_allocated_images.as_ref() {
            assert!(self.pre_allocated_images.as_ref().unwrap().len() == 1, "Error: Number of image handles given is not 1");

            pre_allocated_image = Some(is[0]);
        }

        Image::new(
            c, d,
            self.width.expect("Error: Image builder has no specified width"),
            self.height.expect("Error: Image builder has no specified height"),
            self.depth,
            self.usage.expect("Error: Image builder has no specified usage"),
            self.format.expect("Error: Image builder has no specified format"),
            pre_allocated_image,
        )
    }

    pub unsafe fn build_many(&self, c: &Core, d: &Device, count: usize) -> Vec<Image> {
        if self.pre_allocated_images.is_some() {
            assert!(self.pre_allocated_images.as_ref().unwrap().len() == count, "Error: Number of image handles given is not equal to count");
        }

        let mut images = Vec::<Image>::new();
        for i in 0..count {
            let mut pre_allocated_image: Option<vk::Image> = None;
            if let Some(is) = self.pre_allocated_images.as_ref() {
                pre_allocated_image = Some(is[i]);
            }

            images.push(Image::new(
                c, d,
                self.width.expect("Error: Image builder has no specified width"),
                self.height.expect("Error: Image builder has no specified height"),
                self.depth,
                self.usage.expect("Error: Image builder has no specified usage"),
                self.format.expect("Error: Image builder has no specified format"),
                pre_allocated_image,
            ));
        }

        images
    }
}

impl Image {
    pub unsafe fn new(c: &Core, d: &Device, w: u32, h: u32, de: Option<u32>, u: vk::ImageUsageFlags, format: vk::Format, pre_allocated_image: Option<vk::Image>) -> Image {
        let (image_type, depth) = match de {
            Some(dep) => (vk::ImageType::TYPE_3D, dep),
            None => (vk::ImageType::TYPE_2D, 1),
        };
        
        let extent = vk::Extent3D::builder()
            .width(w)
            .height(h)
            .depth(depth)
            .build();

        let mut image: vk::Image;
        let mut memory: Option<vk::DeviceMemory> = None;

        if let Some(alloced_image) = pre_allocated_image {
            image = alloced_image;
        } else {
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

            image = d.device.create_image(&image_ci, None).unwrap();

            let memory_requirements = d.device.get_image_memory_requirements(image);
            let memory_type_index = d.get_memory_type(c, vk::MemoryPropertyFlags::DEVICE_LOCAL, memory_requirements);
    
            let memory_alloc_i = vk::MemoryAllocateInfo::builder()
                .allocation_size(memory_requirements.size)
                .memory_type_index(memory_type_index as u32);
    
            memory = Some(d.device.allocate_memory(&memory_alloc_i, None).unwrap());
            d.device.bind_image_memory(image, memory.unwrap(), 0).unwrap();
        }

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
            memory: memory,
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