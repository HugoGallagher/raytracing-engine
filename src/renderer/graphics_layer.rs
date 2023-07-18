use ash::vk;

use crate::renderer::core::Core;
use crate::renderer::device::Device;
use crate::renderer::descriptors::DescriptorsBuilder;
use crate::renderer::commands::Commands;
use crate::renderer::graphics_pass::{GraphicsPass, GraphicsPassDrawInfo};
use crate::renderer::buffer::{Buffer, BufferBuilder};
use crate::renderer::vertex_buffer::VertexBuffer;
use crate::renderer::image::{Image, ImageBuilder};
use crate::renderer::push_constant::PushConstantBuilder;
use crate::renderer::vertex_buffer::VertexAttributes;

pub struct GraphicsLayer {
    pub count: usize,

    pub commands: Commands,
    pub passes: Vec<GraphicsPass>,
}

impl GraphicsLayer {
    pub unsafe fn new(c: &Core, d: &Device, count: usize) -> GraphicsLayer {
        let commands = Commands::new(d, d.queue_graphics.1, count, false);

        GraphicsLayer {
            count,
            commands,
            passes: Vec::<GraphicsPass>::new(),
        }
    }

    pub unsafe fn add_pass<T: VertexAttributes>(&mut self, c: &Core, d: &Device, targets: &Vec<Image>, extent: Option<vk::Extent2D>, offset: Option<vk::Offset2D>, verts: Option<&Vec<T>>, indices: Option<&Vec<u32>>, descriptors_builder: Option<DescriptorsBuilder>, push_constant_builder: Option<PushConstantBuilder>, vs: &str, fs: &str, with_depth_buffer: bool, draw_info: GraphicsPassDrawInfo) {
        self.passes.push(GraphicsPass::new(c, d, targets, extent, offset, verts, indices, descriptors_builder, push_constant_builder, vs, fs, with_depth_buffer, draw_info));
    }

    pub unsafe fn fill_push_constant<T>(&mut self, pass_index: usize, data: &T) {
        self.passes[pass_index].push_constant.as_mut().expect("Error: Graphics pass has no push constant to fill").set_data(data);
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