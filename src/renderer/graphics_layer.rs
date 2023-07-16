use std::{ffi::c_void, collections::HashMap};

use ash::vk;

use crate::renderer::core::Core;
use crate::renderer::device::Device;
use crate::renderer::descriptors::{DescriptorsBuilder, DescriptorBindingReference};
use crate::renderer::commands::Commands;
use crate::renderer::graphics_pass::GraphicsPass;
use crate::renderer::buffer::{Buffer, BufferBuilder};
use crate::renderer::image::{Image2D, Image2DBuilder};
use crate::renderer::push_constant::PushConstantBuilder;

pub struct GraphicsLayer {
    pub count: usize,

    pub commands: Commands,
    pub passes: Vec<GraphicsPass>,
}

impl GraphicsLayer {
    pub unsafe fn new(c: &Core, d: &Device, count: usize) -> GraphicsLayer {
        let commands = Commands::new(d, d.queue_graphics.1, count);

        GraphicsLayer {
            count,
            commands,
            passes: Vec::<GraphicsPass>::new(),
        }
    }

    pub unsafe fn add_pass(&mut self, c: &Core, d: &Device, targets: &Vec<Image2D>, descriptors_builder: Option<DescriptorsBuilder>, push_constant_builder: Option<PushConstantBuilder>, vs: &str, fs: &str) {
        self.passes.push(GraphicsPass::new(c, d, targets, descriptors_builder, push_constant_builder, vs, fs));
    }

    pub unsafe fn fill_push_constant<T>(&mut self, pass_index: usize, data: &T) {
        self.passes[pass_index].push_constant.as_mut().expect("Error: Graphics pass has no push constant to fill").set_data(data);
    }

    pub unsafe fn record_one(&self, d: &Device, i: usize, present_index: usize) {
        self.commands.record_one(d, i, |b| {
            for pass in &self.passes {
                let clear_values = [vk::ClearValue { color: vk::ClearColorValue { float32: [0.0, 0.0, 0.0, 0.0]}}];

                let rect = pass.pipeline.scissor;

                let render_pass_bi = vk::RenderPassBeginInfo::builder()
                    .render_pass(pass.pipeline.render_pass)
                    .framebuffer(pass.framebuffers[present_index].framebuffer)
                    .render_area(rect)
                    .clear_values(&clear_values);

                if pass.push_constant.is_some() {
                    d.device.cmd_push_constants(b, pass.pipeline.pipeline_layout, pass.push_constant.as_ref().unwrap().stage, 0, &pass.push_constant.as_ref().unwrap().data);
                }

                if pass.descriptors.is_some() {
                    let descriptors = pass.descriptors.as_ref().unwrap();
                    descriptors.bind(d, &b, vk::PipelineBindPoint::GRAPHICS, &pass.pipeline.pipeline_layout, i);
                }
                
                d.device.cmd_begin_render_pass(b, &render_pass_bi, vk::SubpassContents::INLINE);

                d.device.cmd_bind_pipeline(b, vk::PipelineBindPoint::GRAPHICS, pass.pipeline.pipeline);
                d.device.cmd_set_viewport(b, 0, &[pass.pipeline.viewport]);
                d.device.cmd_set_scissor(b, 0, &[pass.pipeline.scissor]);

                d.device.cmd_draw(b, 6, 1, 0, 0);

                d.device.cmd_end_render_pass(b);
            }
        })
    }
}