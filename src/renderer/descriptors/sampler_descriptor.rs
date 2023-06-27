use ash::version::DeviceV1_0;
use ash::vk;

use crate::renderer::{core::Core, sampler, image};
use crate::renderer::device::Device;
use crate::renderer::image::Image2D;
use crate::renderer::sampler::Sampler;
use crate::renderer::descriptors::Descriptors;
use crate::renderer::swapchain::Swapchain;

pub struct SamplerDescriptor {
    pub samplers: Vec<Sampler>,
}

#[derive(Copy, Clone)]
pub struct SamplerDescriptorBuilder<'a> {
    images: Option<&'a Vec<Image2D>>,
}

impl <'a> SamplerDescriptorBuilder<'a> {
    pub fn new() -> SamplerDescriptorBuilder<'a> {
        SamplerDescriptorBuilder {
            images: None,
        }
    }

    pub fn images(&self, images: &'a Vec<Image2D>) -> SamplerDescriptorBuilder<'a> {
        SamplerDescriptorBuilder {
            images: Some(images),
        }
    }

    pub unsafe fn build(&self, c: &Core, d: &Device, binding: u32, sets: &Vec<vk::DescriptorSet>) -> SamplerDescriptor {
        if self.images.is_none() {
            panic!("Error: Sampler descriptor builder has no images");
        }
        
        let mut samplers = Vec::<Sampler>::new();

        for image in self.images.unwrap() {
            samplers.push(Sampler::new(c, d, image.view));
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