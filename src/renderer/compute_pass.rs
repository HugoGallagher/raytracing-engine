use ash::vk;

use crate::renderer::{core::Core, descriptors::CreationReference, renderer_data::RendererData, layer::Pass};
use crate::renderer::device::Device;
use crate::renderer::descriptors::{Descriptors, DescriptorsBuilder};
use crate::renderer::compute_pipeline::ComputePipeline;
use crate::renderer::push_constant::{PushConstant, PushConstantBuilder};

pub struct ComputePassDispatchInfo {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

pub struct ComputePassBuilder<'a> {
    dispatch_info: Option<ComputePassDispatchInfo>,
    cs: Option<&'a str>,
    push_constant_builder: Option<PushConstantBuilder>,
    descriptors_builder: Option<DescriptorsBuilder>,
}

pub struct ComputePass {
    pub push_constant: Option<PushConstant>,
    pub descriptors: Option<Descriptors>,
    pub pipeline: ComputePipeline,
    pub dispatch_info: ComputePassDispatchInfo,
}

impl Pass for ComputePass {}

impl ComputePassDispatchInfo {
    pub fn new(x: u32, y: u32, z: u32) -> ComputePassDispatchInfo {
        ComputePassDispatchInfo { x, y, z }
    }
}

impl <'a> ComputePassBuilder<'a> {
    pub fn new() -> ComputePassBuilder<'a> {
        ComputePassBuilder {
            dispatch_info: None,
            cs: None,
            push_constant_builder: None,
            descriptors_builder: None,
        }
    }

    pub fn dispatch_info(mut self, dispatch_info: ComputePassDispatchInfo) -> ComputePassBuilder<'a> {
        self.dispatch_info = Some(dispatch_info);

        self
    }

    pub fn compute_shader(mut self, cs: &'a str) -> ComputePassBuilder<'a> {
        self.cs = Some(cs);

        self
    }

    pub fn push_constant<T>(mut self) -> ComputePassBuilder<'a> {
        self.push_constant_builder = Some(PushConstantBuilder::new().stage(vk::ShaderStageFlags::COMPUTE).size(std::mem::size_of::<T>()));

        self
    }

    pub fn descriptors_builder(mut self, descriptors_builder: DescriptorsBuilder) -> ComputePassBuilder<'a> {
        self.descriptors_builder = Some(descriptors_builder.stage(vk::ShaderStageFlags::COMPUTE));

        self
    }

    pub fn descriptors(mut self, create_refs: Vec<CreationReference>, data: &RendererData) -> ComputePassBuilder<'a> {
        let mut descriptors_builder = DescriptorsBuilder::new()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .count(data.count);

        for create_ref in create_refs {
            match create_ref {
                CreationReference::Uniform(name) => { descriptors_builder = descriptors_builder.add_uniform_simple(data.get_buffers(&name)); },
                CreationReference::Storage(name) => { descriptors_builder = descriptors_builder.add_storage_simple(data.get_buffers(&name)); },
                CreationReference::Image(name) => { descriptors_builder = descriptors_builder.add_image_simple(data.get_images(&name)); },
                CreationReference::Sampler(name) => { descriptors_builder = descriptors_builder.add_sampler_simple(data.get_images(&name)); },
            }
        }

        self.descriptors_builder = Some(descriptors_builder);
        
        self
    }

    pub unsafe fn build(self, c: &Core, d: &Device) -> ComputePass {
        ComputePass::new(c, d, self.descriptors_builder, self.push_constant_builder, self.cs.expect("Error: Compute pass builder has no compute shader"), self.dispatch_info.expect("Error: Compute pass builder has no dispatch info"))
    }
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