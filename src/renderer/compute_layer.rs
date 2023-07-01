use std::ffi::c_void;

use ash::vk;
use ash::version::DeviceV1_0;

use crate::renderer::core::Core;
use crate::renderer::device::Device;
use crate::renderer::descriptors::DescriptorsBuilder;
use crate::renderer::commands::Commands;
use crate::renderer::compute_pass::ComputePass;

pub struct ComputeLayer {
    pub commands: Commands,
    pub passes: Vec<ComputePass>,
}

impl ComputeLayer {
    pub unsafe fn new(c: &Core, d: &Device, count: usize) -> ComputeLayer {
        let commands = Commands::new(d, d.queue_compute.1, count);

        ComputeLayer {
            commands,
            passes: Vec::<ComputePass>::new(),
        }
    }

    pub unsafe fn add_pass(&mut self, c: &Core, d: &Device, descriptors_builder: Option<DescriptorsBuilder>, push_constant_size: Option<usize>, cs: &str, workgroups: (u32, u32, u32)) {
        self.passes.push(ComputePass::new(c, d, descriptors_builder, push_constant_size, cs, workgroups));
    }

    pub unsafe fn fill_push_constant<T>(&mut self, pass_index: usize, data: &T) {
        self.passes[pass_index].push_constant.as_mut().expect("Error: Compute pass has no push constant to fill").set_data(data);
    }

    pub unsafe fn record_one(&self, d: &Device, i: usize) {
        self.commands.record_one(d, i, |b| {
            for pass in &self.passes {
                d.device.cmd_bind_pipeline(b, vk::PipelineBindPoint::COMPUTE, pass.pipeline.pipeline);

                if pass.push_constant.is_some() {
                    d.device.cmd_push_constants(b, pass.pipeline.pipeline_layout, vk::ShaderStageFlags::COMPUTE, 0, &pass.push_constant.as_ref().unwrap().data);
                }

                if pass.descriptors.is_some() {
                    let descriptors = pass.descriptors.as_ref().unwrap();
                    descriptors.bind(d, &b, vk::PipelineBindPoint::COMPUTE, &pass.pipeline.pipeline_layout, i);
                }

                d.device.cmd_dispatch(b, pass.workgroups.0, pass.workgroups.1, pass.workgroups.2);
            }
        })
    }
}