use ash::vk;

use crate::renderer::core::Core;
use crate::renderer::device::Device;
use crate::renderer::descriptors::{Descriptors, DescriptorsBuilder};
use crate::renderer::compute_pipeline::ComputePipeline;
use crate::renderer::push_constant::{PushConstant, PushConstantBuilder};

pub struct ComputePassDispatchInfo {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

pub struct ComputePass {
    pub push_constant: Option<PushConstant>,
    pub descriptors: Option<Descriptors>,
    pub pipeline: ComputePipeline,
    pub dispatch_info: ComputePassDispatchInfo,
}

impl ComputePass {
    pub unsafe fn new(c: &Core, d: &Device, descriptors_builder: Option<DescriptorsBuilder>, push_constant_builder: Option<PushConstantBuilder>, cs: &str, dispatch_info: ComputePassDispatchInfo) -> ComputePass {
        let descriptors = match descriptors_builder {
            Some(de_b) => Some(de_b.build(c, d)),
            None => None
        };

        let descriptor_set_layout = match descriptors.as_ref() {
            Some(de) => Some(de.set_layout),
            None => None
        };

        let push_constant = match push_constant_builder {
            Some(builder) => Some(builder.build()),
            None => None
        };
        
        let pipeline = ComputePipeline::new(c, d, descriptor_set_layout, push_constant.as_ref(), cs);

        ComputePass {
            push_constant,
            descriptors,
            pipeline,
            dispatch_info,
        }
    }
}