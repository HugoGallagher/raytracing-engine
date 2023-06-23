use ash::version::DeviceV1_0;
use ash::vk;

use crate::renderer::core::Core;
use crate::renderer::device::Device;
use crate::renderer::sampler::Sampler;
use crate::renderer::descriptors::Descriptors;
use crate::renderer::swapchain::Swapchain;

pub struct SamplerDescriptor {
    pub samplers: Vec<Sampler>,
}

pub struct SamplerDescriptorBuilder {
    
}

impl SamplerDescriptor {
    pub unsafe fn new(p: &Descriptors, b: u32, sms: &Vec<Sampler>, c: &Core, d: &Device, s: &Swapchain) -> SamplerDescriptor {
        let mut write_sets = Vec::<vk::WriteDescriptorSet>::new();

        for i in 0..s.image_count {
            let image_is = [vk::DescriptorImageInfo::builder()
                .sampler(sms[i as usize].sampler)
                .image_view(sms[i as usize].view)
                .image_layout(vk::ImageLayout::GENERAL)
                .build()];

            let write_set = vk::WriteDescriptorSet::builder()
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .dst_binding(b)
                .dst_set(p.sets[i as usize])
                .image_info(&image_is)
                .build();

            write_sets.push(write_set);
        }

        d.device.update_descriptor_sets(&write_sets, &[]);

        SamplerDescriptor {
            samplers: sms.clone(),
        }
    }
}