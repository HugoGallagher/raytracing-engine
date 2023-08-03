use ash::vk;

use crate::renderer::core::Core;
use crate::renderer::device::Device;

#[derive(Copy, Clone)]
pub struct Sampler {
    pub sampler: vk::Sampler,
    pub view: vk::ImageView,
}

impl Sampler {
    pub unsafe fn new(c: &Core, d: &Device, v: vk::ImageView) -> Sampler {
        let sampler_ci = vk::SamplerCreateInfo::builder()
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_BORDER)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_BORDER)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_BORDER)
            .mag_filter(vk::Filter::NEAREST)
            .min_filter(vk::Filter::NEAREST);
        
        let sampler = d.device.create_sampler(&sampler_ci, None).unwrap();

        Sampler {
            sampler,
            view: v,
        }
    }
}