use ash::version::DeviceV1_0;
use ash::vk;

use crate::renderer::{core::Core, sampler, image};
use crate::renderer::device::Device;
use crate::renderer::image::Image2D;
use crate::renderer::sampler::Sampler;
use crate::renderer::descriptors::Descriptors;
use crate::renderer::swapchain::Swapchain;

struct ImageData {
    image: vk::Image,
    view: vk::ImageView,
}

pub struct SamplerDescriptor {
    pub samplers: Vec<Sampler>,
}

pub struct SamplerDescriptorBuilder {
    image_datas: Option<Vec<ImageData>>,
}

impl SamplerDescriptorBuilder {
    pub fn new() -> SamplerDescriptorBuilder {
        SamplerDescriptorBuilder {
            image_datas: None,
        }
    }

    pub fn images(&self, images: &Vec<Image2D>) -> SamplerDescriptorBuilder {
        let image_datas = images.iter().map(|image| { ImageData { image: image.image, view: image.view} }).collect();
        SamplerDescriptorBuilder {
            image_datas: Some(image_datas),
        }
    }

    pub unsafe fn build(&self, c: &Core, d: &Device, binding: u32, sets: &Vec<vk::DescriptorSet>) -> SamplerDescriptor {
        if self.image_datas.is_none() {
            panic!("Error: Sampler descriptor builder has no images");
        }
        
        let mut samplers = Vec::<Sampler>::new();

        for image_data in self.image_datas.as_ref().unwrap() {
            samplers.push(Sampler::new(c, d, image_data.view));
        }

        SamplerDescriptor::new(c, d, binding, &samplers, sets)
    }
}

impl SamplerDescriptor {
    pub unsafe fn new(c: &Core, d: &Device, binding: u32, samplers: &Vec<Sampler>, sets: &Vec<vk::DescriptorSet>) -> SamplerDescriptor {
        let mut write_sets = Vec::<vk::WriteDescriptorSet>::new();

        for i in 0..samplers.len() {
            let image_is = [vk::DescriptorImageInfo::builder()
                .sampler(samplers[i as usize].sampler)
                .image_view(samplers[i as usize].view)
                .image_layout(vk::ImageLayout::GENERAL)
                .build()];

            let write_set = vk::WriteDescriptorSet::builder()
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .dst_binding(binding)
                .dst_set(sets[i as usize])
                .image_info(&image_is)
                .build();

            write_sets.push(write_set);
        }

        d.device.update_descriptor_sets(&write_sets, &[]);

        SamplerDescriptor {
            samplers: samplers.clone(),
        }
    }
}