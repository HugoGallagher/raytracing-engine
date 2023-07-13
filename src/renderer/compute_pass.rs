use ash::vk;

use crate::renderer::core::Core;
use crate::renderer::device::Device;
use crate::renderer::descriptors::{Descriptors, DescriptorsBuilder};
use crate::renderer::compute_pipeline::ComputePipeline;
use crate::renderer::push_constant::PushConstant;

pub struct ComputePass {
    pub push_constant: Option<PushConstant>,
    pub descriptors: Option<Descriptors>,
    pub pipeline: ComputePipeline,
    pub workgroups: (u32, u32, u32),
}

impl ComputePass {
    pub unsafe fn new(c: &Core, d: &Device, descriptors_builder: Option<DescriptorsBuilder>, push_constant_size: Option<usize>, cs: &str, workgroups: (u32, u32, u32)) -> ComputePass {
        let descriptors = match descriptors_builder {
            Some(de_b) => Some(de_b.build(c, d)),
            None => None
        };

        let descriptor_set_layout = match descriptors.as_ref() {
            Some(de) => Some(de.set_layout),
            None => None
        };

        let push_constant = match push_constant_size {
            Some(size) => Some(PushConstant::new(size)),
            None => None
        };
        
        let pipeline = ComputePipeline::new(c, d, descriptor_set_layout, push_constant_size, cs);

        ComputePass {
            push_constant,
            descriptors,
            pipeline,
            workgroups,
        }
    }
}