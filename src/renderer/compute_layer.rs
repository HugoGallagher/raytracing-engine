use std::{ffi::c_void, collections::HashMap};

use ash::vk;

use crate::renderer::{core::Core, semaphore::Semaphore};
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
    pub pass_refs: HashMap<String, usize>,

    pub semaphore: Semaphore,
}

impl ComputeLayer {
    pub unsafe fn new(c: &Core, d: &Device, count: usize) -> ComputeLayer {
        let commands = Commands::new(d, d.queue_compute.1, count, false);
        let semaphore = Semaphore::new(d);

        ComputeLayer {
            count,
            commands,
            passes: Vec::new(),
            pass_refs: HashMap::new(),
            semaphore,
        }
    }

    pub unsafe fn add_pass(&mut self, name: &str, pass: ComputePass) {
        self.passes.push(pass);
        self.pass_refs.insert(name.to_string(), self.passes.len() - 1);
    }

    pub fn get_pass(&self, name: &str) -> &ComputePass {
        &self.passes[*self.pass_refs.get(name).unwrap()]
    }

    pub unsafe fn fill_push_constant<T>(&mut self, name: &str, data: &T) {
        self.passes[*self.pass_refs.get(name).unwrap()].push_constant.as_mut().expect("Error: Compute pass has no push constant to fill").set_data(data);
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