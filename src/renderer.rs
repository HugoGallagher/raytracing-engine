pub mod core;
pub mod device;
pub mod swapchain;
pub mod buffer;
pub mod image;
pub mod sampler;
pub mod vertex_buffer;
pub mod descriptors;
pub mod shader;
pub mod framebuffer;
pub mod commands;
pub mod compute_pipeline;
pub mod graphics_pipeline;
pub mod compute_pass;
pub mod graphics_pass;
pub mod compute_layer;
pub mod graphics_layer;
pub mod fence;
pub mod semaphore;
pub mod frame;
pub mod mesh;
pub mod push_constant;

use std::{mem, ffi::c_void, collections::HashMap};

use ash::vk;
use raw_window_handle::{RawWindowHandle, RawDisplayHandle};

use crate::{math::{vec::{Vec4, Vec3, Vec2}, mat::Mat4}, renderer::{mesh::FromObjTri, vertex_buffer::VertexAttributes, compute_layer::ComputeLayer}, util::graph::{Graph, self}};

pub enum LayerRef {
    Compute(usize),
    Graphics(usize),
}

#[derive(Copy, Clone)]
pub struct LayerDependencyInfo {
    stage: vk::PipelineStageFlags,
}

pub struct LayerSubmitInfo {
    wait_semaphores: Vec<vk::Semaphore>,
    wait_stages: Vec<vk::PipelineStageFlags>,
    signal_semaphores: Vec<vk::Semaphore>,
    command_buffers: Vec<vk::CommandBuffer>,
    queue: vk::Queue,
    fence: vk::Fence,
    submit_i: vk::SubmitInfo,
}

pub struct Renderer {
    pub core: core::Core,
    pub device: device::Device,
    pub swapchain: swapchain::Swapchain,
 
    pub buffers: HashMap<String, Vec<buffer::Buffer>>,
    pub images: HashMap<String, Vec<image::Image>>,
 
    pub compute_layers: Vec<compute_layer::ComputeLayer>,
    pub graphics_layers: Vec<graphics_layer::GraphicsLayer>,
 
    pub layers: Graph<LayerRef, LayerDependencyInfo>,
 
    pub frames: Vec<frame::Frame>,
 
    pub frames_in_flight: usize,
    pub current_frame: usize,
    pub present_index: usize,
}

impl Renderer {
    pub unsafe fn new(window: RawWindowHandle, display: RawDisplayHandle) -> Renderer {
        const FRAMES_IN_FLIGHT: u32 = 2;

        let debug = true;

        let core = core::Core::new(debug, display);
        let device = device::Device::new(&core, window, display);
        let swapchain = swapchain::Swapchain::new(&core, &device);

        let layers = Graph::new();

        let compute_layers = Vec::<compute_layer::ComputeLayer>::new();
        let graphics_layers = Vec::<graphics_layer::GraphicsLayer>::new();

        let mut buffers = HashMap::<String, Vec<buffer::Buffer>>::new();
        let mut images = HashMap::<String, Vec<image::Image>>::new();

        let mut frames = Vec::<frame::Frame>::new();
        for _ in 0..FRAMES_IN_FLIGHT {
            frames.push(frame::Frame::new(&device));
        }

        Renderer {
            core,
            device,
            swapchain,

            buffers,
            images,

            layers,

            compute_layers,
            graphics_layers,

            frames,

            frames_in_flight: FRAMES_IN_FLIGHT as usize,
            current_frame: 0,
            present_index: 0,
        }
    }

    pub unsafe fn pre_draw(&mut self) {
        self.current_frame = (self.current_frame + 1) % self.frames_in_flight;

        let active_frame = self.frames[self.current_frame];
        
        self.device.device.wait_for_fences(&[active_frame.in_flight_fence.fence], true, u64::MAX).unwrap();
        self.device.device.reset_fences(&[active_frame.in_flight_fence.fence]).unwrap();
        
        self.present_index = self.swapchain.swapchain_init.acquire_next_image(self.swapchain.swapchain, u64::MAX, active_frame.image_available_semaphore.semaphore, vk::Fence::null()).unwrap().0 as usize;
    }

    pub unsafe fn draw(&mut self) {
        let active_frame = self.frames[self.current_frame];

        let present_indices = [self.present_index as u32];

        for layer in &self.compute_layers {
            layer.record_one(&self.device, self.current_frame);
        }

        for layer in &self.graphics_layers {
            layer.record_one(&self.device, self.current_frame, self.present_index);
        }

        let mut present_wait_semaphores = Vec::<vk::Semaphore>::new();

        let mut layer_submit_infos = Vec::<LayerSubmitInfo>::with_capacity(self.layers.node_count());

        let mut nodes = self.layers.breadth_first_backwards("final_layer");
        nodes.reverse();

        let mut present_info_set = false;
        for node in nodes {
            let mut wait_semaphores = Vec::<vk::Semaphore>::new();
            let mut wait_stages = Vec::<vk::PipelineStageFlags>::new();
            let mut signal_semaphores = Vec::<vk::Semaphore>::new();

            let dependencies = self.layers.get_prev_edges(&node.name);
            let dependants = self.layers.get_next_edges(&node.name);

            for dependency in dependencies {
                let layer_ref = &self.layers.get_node(&dependency.src).data;

                wait_semaphores.push(match layer_ref {
                    LayerRef::Compute(i) => self.get_compute_layer(&dependency.src).semaphore.semaphore,
                    LayerRef::Graphics(i) => self.get_graphics_layer(&dependency.src).semaphore.semaphore,
                });
                wait_stages.push(dependency.info.stage);
            }

            signal_semaphores.push(match node.data {
                LayerRef::Compute(i) => self.get_compute_layer(&node.name).semaphore.semaphore,
                LayerRef::Graphics(i) => self.get_graphics_layer(&node.name).semaphore.semaphore,
            });

            let mut fence = vk::Fence::null();

            if let LayerRef::Graphics(i) = node.data {
                let layer = self.get_graphics_layer(&node.name);
                if layer.present {
                    assert!(!present_info_set, "Error: Multiple graphics layers marked as present");
                    present_info_set = true;

                    wait_semaphores.push(active_frame.image_available_semaphore.semaphore);
                    wait_stages.push(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT);
                    fence = active_frame.in_flight_fence.fence;
                    present_wait_semaphores = signal_semaphores.clone();
                }
            }

            let (command_buffers, queue) = match node.data {
                LayerRef::Compute(i) => (vec![self.get_compute_layer(&node.name).commands.buffers[self.current_frame]], self.device.queue_compute.0),
                LayerRef::Graphics(i) => (vec![self.get_graphics_layer(&node.name).commands.buffers[self.current_frame]], self.device.queue_graphics.0),
            };

            let mut layer_submit_info = LayerSubmitInfo {
                wait_semaphores,
                wait_stages,
                signal_semaphores,
                command_buffers,
                queue,
                fence,
                submit_i: vk::SubmitInfo::builder().build(),
            };

            layer_submit_info.submit_i = vk::SubmitInfo::builder()
                .wait_semaphores(&layer_submit_info.wait_semaphores)
                .signal_semaphores(&layer_submit_info.signal_semaphores)
                .wait_dst_stage_mask(&layer_submit_info.wait_stages)
                .command_buffers(&layer_submit_info.command_buffers)
                .build();

            layer_submit_infos.push(layer_submit_info);
        };
        
        for layer_submit_info in layer_submit_infos {
            self.device.device.queue_submit(layer_submit_info.queue, &[layer_submit_info.submit_i], layer_submit_info.fence).unwrap();
        }

        let swapchains = [self.swapchain.swapchain];

        let present_i = vk::PresentInfoKHR::builder()
            .wait_semaphores(&present_wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&present_indices);

        self.swapchain.swapchain_init.queue_present(self.device.queue_present.0, &present_i).unwrap();
    }

    pub unsafe fn add_buffer(&mut self, name: &str, builder: buffer::BufferBuilder) {
        self.buffers.insert(name.to_string(), builder.build_many(&self.core, &self.device, self.frames_in_flight));
    }

    pub unsafe fn add_image(&mut self, name: &str, builder: image::ImageBuilder) {
        self.images.insert(name.to_string(), builder.build_many(&self.core, &self.device, self.frames_in_flight));
    }

    pub unsafe fn add_compute_layer(&mut self, name: &str) {
        self.compute_layers.push(compute_layer::ComputeLayer::new(&self.core, &self.device, self.frames_in_flight));
        self.layers.add_node(name, LayerRef::Compute(self.compute_layers.len() - 1));
    }

    pub unsafe fn add_graphics_layer(&mut self, name: &str, present: bool) {
        self.graphics_layers.push(graphics_layer::GraphicsLayer::new(&self.core, &self.device, self.frames_in_flight, present));
        self.layers.add_node(name, LayerRef::Graphics(self.graphics_layers.len() - 1));
    }

    pub unsafe fn add_layer_dependency(&mut self, src: &str, dst: &str, stage: vk::PipelineStageFlags) {
        self.layers.add_edge(src, dst, LayerDependencyInfo { stage });
    }

    pub unsafe fn add_compute_pass(&mut self, layer_name: &str, pass_name: &str, builder: compute_pass::ComputePassBuilder) {
        let layer_ref = &self.layers.get_node(layer_name).data;

        match layer_ref {
            LayerRef::Compute(i) => { self.compute_layers[*i].add_pass(pass_name, builder.build(&self.core, &self.device)); }
            _ => panic!("Error: Layer is not a compute layer")
        }
    }

    pub unsafe fn add_graphics_pass<T: VertexAttributes>(&mut self, layer_name: &str, pass_name: &str, builder: graphics_pass::GraphicsPassBuilder<T>) {
        let layer_ref = &self.layers.get_node(layer_name).data;

        match layer_ref {
            LayerRef::Graphics(i) => { self.graphics_layers[*i].add_pass(pass_name, builder.build(&self.core, &self.device)); }
            _ => panic!("Error: Layer is not a compute layer")
        }
    }

    pub fn get_compute_layer(&self, name: &str) -> &compute_layer::ComputeLayer {
        let layer_ref = &self.layers.get_node(name).data;
        if let LayerRef::Compute(i) = layer_ref {
            &self.compute_layers[*i]
        } else {
            panic!("Error: No compute layer with that name exists")
        }
    }

    pub fn get_graphics_layer(&self, name: &str) -> &graphics_layer::GraphicsLayer {
        let layer_ref = &self.layers.get_node(name).data;
        if let LayerRef::Graphics(i) = layer_ref {
            &self.graphics_layers[*i]
        } else {
            panic!("Error: No graphics layer with that name exists")
        }
    }

    pub fn get_compute_layer_mut(&mut self, name: &str) -> &mut compute_layer::ComputeLayer {
        let layer_ref = &self.layers.get_node(name).data;
        if let LayerRef::Compute(i) = layer_ref {
            &mut self.compute_layers[*i]
        } else {
            panic!("Error: No compute layer with that name exists")
        }
    }

    pub fn get_graphics_layer_mut(&mut self, name: &str) -> &mut graphics_layer::GraphicsLayer {
        let layer_ref = &self.layers.get_node(name).data;
        if let LayerRef::Graphics(i) = layer_ref {
            &mut self.graphics_layers[*i]
        } else {
            panic!("Error: No graphics layer with that name exists")
        }
    }

    pub unsafe fn fill_buffer<T>(&mut self, name: &str, data: &Vec<T>) {
        self.buffers.get(name).unwrap()[self.current_frame].fill(&self.device, &data);
    }

    pub unsafe fn fill_push_constant<T>(&mut self, layer_name: &str, pass_name: &str, data: &T) {
        let layer_ref = &self.layers.get_node(layer_name).data;
        match layer_ref {
            LayerRef::Compute(i) => self.get_compute_layer_mut(layer_name).fill_push_constant(pass_name, data),
            LayerRef::Graphics(i) => self.get_graphics_layer_mut(layer_name).fill_push_constant(pass_name, data),
        }
    }
}