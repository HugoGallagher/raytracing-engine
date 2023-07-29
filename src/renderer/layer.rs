use ash::vk;

use std::collections::HashMap;

use crate::renderer::{core::Core, graphics_pass::GraphicsPassBuilder, semaphore::Semaphore, compute_pass::ComputePass};
use crate::renderer::device::Device;
use crate::renderer::descriptors::DescriptorsBuilder;
use crate::renderer::commands::Commands;
use crate::renderer::graphics_pass::{GraphicsPass, GraphicsPassDrawInfo};
use crate::renderer::buffer::{Buffer, BufferBuilder};

#[derive(Copy, Clone)]
pub enum LayerExecution {
    Main,
    Async,
}

#[derive(Copy, Clone)]
pub enum PassType {
    Compute,
    Graphics,
}

#[derive(Copy, Clone)]
pub struct PassRef {
    pass_type: PassType,
    index: usize,
}

#[derive(Copy, Clone)]
pub struct LayerDependencyInfo {
    pub stage: vk::PipelineStageFlags,
}

pub trait Pass {}

pub struct LayerSubmitInfo {
    pub wait_semaphores: Vec<vk::Semaphore>,
    pub wait_stages: Vec<vk::PipelineStageFlags>,
    pub signal_semaphores: Vec<vk::Semaphore>,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub queue: vk::Queue,
    pub fence: vk::Fence,
    pub submit_i: vk::SubmitInfo,
}

pub struct Layer {
    pub count: usize,

    pub commands: Commands,
    pub exec: LayerExecution,

    pub graphics_passes: Vec<GraphicsPass>,
    pub compute_passes: Vec<ComputePass>,

    pub pass_refs: HashMap<String, PassRef>,

    pub semaphore: Semaphore,

    pub present: bool,
}

impl Layer {
    pub unsafe fn new(c: &Core, d: &Device, count: usize, present: bool, exec: LayerExecution) -> Layer {
        let commands = Commands::new(d, d.get_queue(exec).1, count, false);
        let semaphore = Semaphore::new(d);

    Layer {
            count,
            commands,
            exec,
            graphics_passes: Vec::new(),
            compute_passes: Vec::new(),
            pass_refs: HashMap::new(),
            semaphore,
            present,
        }
    }

    pub unsafe fn add_compute_pass(&mut self, name: &str, pass: ComputePass) {
        self.compute_passes.push(pass);
        self.pass_refs.insert(name.to_string(), PassRef { pass_type: PassType::Compute, index: self.compute_passes.len() - 1 });
    }

    pub unsafe fn add_graphics_pass(&mut self, name: &str, pass: GraphicsPass) {
        self.graphics_passes.push(pass);
        self.pass_refs.insert(name.to_string(), PassRef { pass_type: PassType::Graphics, index: self.graphics_passes.len() - 1 });
    }

    pub fn get_compute_pass(&self, name: &str) -> &ComputePass {
        &self.compute_passes[self.pass_refs.get(name).unwrap().index]
    }

    pub fn get_graphics_pass(&self, name: &str) -> &GraphicsPass {
        &self.graphics_passes[self.pass_refs.get(name).unwrap().index]
    }

    pub unsafe fn fill_compute_push_constant<T>(&mut self, name: &str, data: &T) {
        self.compute_passes[self.pass_refs.get(name).unwrap().index].push_constant.as_mut().expect("Error: Graphics pass has no vertex push constant to fill").set_data(data);
    }

    pub unsafe fn fill_vertex_push_constant<T>(&mut self, name: &str, data: &T) {
        self.graphics_passes[self.pass_refs.get(name).unwrap().index].vertex_push_constant.as_mut().expect("Error: Graphics pass has no vertex push constant to fill").set_data(data);
    }

    pub unsafe fn fill_fragment_push_constant<T>(&mut self, name: &str, data: &T) {
        self.graphics_passes[self.pass_refs.get(name).unwrap().index].fragment_push_constant.as_mut().expect("Error: Graphics pass has no fragment push constant to fill").set_data(data);
    }

    pub unsafe fn record_one(&self, d: &Device, i: usize, present_index: usize) {
        let mut pass_list = Vec::<PassRef>::with_capacity(self.compute_passes.len() + self.graphics_passes.len());

        for i in 0..self.compute_passes.len() {
            pass_list.push(PassRef { pass_type: PassType::Compute, index: i });
        }
        for i in 0..self.graphics_passes.len() {
            pass_list.push(PassRef { pass_type: PassType::Graphics, index: i });
        }

        self.commands.record_one(d, i, |b| {
            for pass_ref in &pass_list {
                match pass_ref.pass_type {
                    PassType::Compute => {
                        let pass = &self.compute_passes[pass_ref.index];
                        
                        d.device.cmd_bind_pipeline(b, vk::PipelineBindPoint::COMPUTE, pass.pipeline.pipeline);
        
                        if pass.push_constant.is_some() {
                            d.device.cmd_push_constants(b, pass.pipeline.pipeline_layout, pass.push_constant.as_ref().unwrap().stage, 0, &pass.push_constant.as_ref().unwrap().data);
                        }
        
                        if pass.descriptors.is_some() {
                            let descriptors = pass.descriptors.as_ref().unwrap();
                            descriptors.bind(d, &b, vk::PipelineBindPoint::COMPUTE, &pass.pipeline.pipeline_layout, i);
                        }
        
                        d.device.cmd_dispatch(b, pass.dispatch_info.x, pass.dispatch_info.y, pass.dispatch_info.z);
                    },
                    PassType::Graphics => {
                        let pass = &self.graphics_passes[pass_ref.index];

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
                }
            }
        })
    }
}