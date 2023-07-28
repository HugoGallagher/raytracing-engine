use std::collections::HashMap;

use ash::vk;

use crate::renderer::{core::Core, graphics_pass::GraphicsPassBuilder, semaphore::Semaphore};
use crate::renderer::device::Device;
use crate::renderer::descriptors::DescriptorsBuilder;
use crate::renderer::commands::Commands;
use crate::renderer::graphics_pass::{GraphicsPass, GraphicsPassDrawInfo};
use crate::renderer::buffer::{Buffer, BufferBuilder};

pub struct GraphicsLayer {
    pub count: usize,

    pub commands: Commands,
    pub passes: Vec<GraphicsPass>,
    pub pass_refs: HashMap<String, usize>,

    pub semaphore: Semaphore,

    pub present: bool,
}

impl GraphicsLayer {
    pub unsafe fn new(c: &Core, d: &Device, count: usize, present: bool) -> GraphicsLayer {
        let commands = Commands::new(d, d.queue_graphics.1, count, false);
        let semaphore = Semaphore::new(d);

        GraphicsLayer {
            count,
            commands,
            passes: Vec::new(),
            pass_refs: HashMap::new(),
            semaphore,
            present,
        }
    }

    pub unsafe fn add_pass(&mut self, name: &str, pass: GraphicsPass) {
        self.passes.push(pass);
        self.pass_refs.insert(name.to_string(), self.passes.len() - 1);
    }

    pub fn get_pass(&self, name: &str) -> &GraphicsPass {
        &self.passes[*self.pass_refs.get(name).unwrap()]
    }

    pub unsafe fn fill_vertex_push_constant<T>(&mut self, name: &str, data: &T) {
        self.passes[*self.pass_refs.get(name).unwrap()].vertex_push_constant.as_mut().expect("Error: Graphics pass has no vertex push constant to fill").set_data(data);
    }

    pub unsafe fn fill_fragment_push_constant<T>(&mut self, name: &str, data: &T) {
        self.passes[*self.pass_refs.get(name).unwrap()].fragment_push_constant.as_mut().expect("Error: Graphics pass has no fragment push constant to fill").set_data(data);
    }

    pub unsafe fn record_one(&self, d: &Device, i: usize, present_index: usize) {
        self.commands.record_one(d, i, |b| {
            for pass in &self.passes {
                let mut clear_values = vec![vk::ClearValue { color: vk::ClearColorValue { float32: [0.0, 0.0, 0.0, 0.0]}}];

                if pass.pipeline.depth_image.is_some() {
                    clear_values.push(vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0} });
                }

                let rect = pass.pipeline.scissor;

                let render_pass_bi = vk::RenderPassBeginInfo::builder()
                    .render_pass(pass.pipeline.render_pass)
                    .framebuffer(pass.framebuffers[present_index].framebuffer)
                    .render_area(rect)
                    .clear_values(&clear_values);

                if let Some(push_constant) = &pass.vertex_push_constant {
                    d.device.cmd_push_constants(b, pass.pipeline.pipeline_layout, push_constant.stage, 0, &push_constant.data);
                }

                if let Some(push_constant) = &pass.fragment_push_constant {
                    d.device.cmd_push_constants(b, pass.pipeline.pipeline_layout, push_constant.stage, 0, &push_constant.data);
                }

                if let Some(descriptors) = &pass.vertex_descriptors {
                    descriptors.bind(d, &b, vk::PipelineBindPoint::GRAPHICS, &pass.pipeline.pipeline_layout, i);
                }

                if let Some(descriptors) = &pass.fragment_descriptors {
                    descriptors.bind(d, &b, vk::PipelineBindPoint::GRAPHICS, &pass.pipeline.pipeline_layout, i);
                }
                
                d.device.cmd_begin_render_pass(b, &render_pass_bi, vk::SubpassContents::INLINE);

                d.device.cmd_bind_pipeline(b, vk::PipelineBindPoint::GRAPHICS, pass.pipeline.pipeline);
                
                d.device.cmd_set_viewport(b, 0, &[pass.pipeline.viewport]);
                d.device.cmd_set_scissor(b, 0, &[pass.pipeline.scissor]);

                if pass.vertex_buffer.is_some() {
                    d.device.cmd_bind_vertex_buffers(b, 0, &[pass.vertex_buffer.as_ref().unwrap().buffer.buffer], &[0]);
                }
                
                if pass.indexed {
                    d.device.cmd_bind_index_buffer(b, pass.vertex_buffer.as_ref().unwrap().index_buffer.unwrap().buffer, 0, vk::IndexType::UINT32);
                    d.device.cmd_draw_indexed(b, pass.draw_info.index_count, pass.draw_info.instance_count, pass.draw_info.first_vertex, pass.draw_info.vertex_offset, pass.draw_info.instance_count);
                } else {
                    d.device.cmd_draw(b, pass.draw_info.vertex_count, pass.draw_info.instance_count, pass.draw_info.first_vertex, pass.draw_info.instance_count);
                }

                d.device.cmd_end_render_pass(b);
            }
        })
    }
}