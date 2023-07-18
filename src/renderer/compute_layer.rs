use std::{ffi::c_void, collections::HashMap};

use ash::vk;

use crate::renderer::core::Core;
use crate::renderer::device::Device;
use crate::renderer::descriptors::{DescriptorsBuilder, DescriptorBindingReference};
use crate::renderer::push_constant::{PushConstant, PushConstantBuilder};
use crate::renderer::commands::Commands;
use crate::renderer::compute_pass::{ComputePass, ComputePassDispatchInfo};
use crate::renderer::buffer::{Buffer, BufferBuilder};
use crate::renderer::image::{Image, ImageBuilder};

pub struct ComputeLayer {
    pub count: usize,

    pub commands: Commands,
    pub passes: Vec<ComputePass>,
}

impl ComputeLayer {
    pub unsafe fn new(c: &Core, d: &Device, count: usize) -> ComputeLayer {
        let commands = Commands::new(d, d.queue_compute.1, count, false);

        ComputeLayer {
            count,
            commands,
            passes: Vec::<ComputePass>::new(),
        }
    }

    pub unsafe fn add_pass(&mut self, pass: ComputePass) {
        self.passes.push(pass);
    }

    pub unsafe fn fill_push_constant<T>(&mut self, pass_index: usize, data: &T) {
        self.passes[pass_index].push_constant.as_mut().expect("Error: Compute pass has no push constant to fill").set_data(data);
    }

    pub unsafe fn record_one(&self, d: &Device, i: usize) {
        self.commands.record_one(d, i, |b| {
            for pass in &self.passes {
                d.device.cmd_bind_pipeline(b, vk::PipelineBindPoint::COMPUTE, pass.pipeline.pipeline);

                if pass.push_constant.is_some() {
                    d.device.cmd_push_constants(b, pass.pipeline.pipeline_layout, pass.push_constant.as_ref().unwrap().stage, 0, &pass.push_constant.as_ref().unwrap().data);
                }

                if pass.descriptors.is_some() {
                    let descriptors = pass.descriptors.as_ref().unwrap();
                    descriptors.bind(d, &b, vk::PipelineBindPoint::COMPUTE, &pass.pipeline.pipeline_layout, i);
                }

                d.device.cmd_dispatch(b, pass.dispatch_info.x, pass.dispatch_info.y, pass.dispatch_info.z);
            }
        })
    }
}