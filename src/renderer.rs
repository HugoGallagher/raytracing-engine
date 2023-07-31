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
pub mod fence;
pub mod semaphore;
pub mod frame;
pub mod mesh;
pub mod push_constant;
pub mod renderer_data;
pub mod layer;

use ash::vk;
use raw_window_handle::{RawWindowHandle, RawDisplayHandle};

use crate::{renderer::{vertex_buffer::VertexAttributes, buffer::Buffer, image::Image, layer::{LayerDependencyInfo, LayerSubmitInfo, PassDependency}}, util::graph::Graph};

pub struct Renderer {
    pub core: core::Core,
    pub device: device::Device,
    pub swapchain: swapchain::Swapchain,

    pub data: renderer_data::RendererData,
 
    pub layers: Vec<layer::Layer>,
    pub layer_graph: Graph<usize, LayerDependencyInfo>,
 
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

        let layers = Vec::<layer::Layer>::new();
        let layer_graph = Graph::new();

        let data = renderer_data::RendererData::new(FRAMES_IN_FLIGHT as usize);

        let mut frames = Vec::<frame::Frame>::new();
        for _ in 0..FRAMES_IN_FLIGHT {
            frames.push(frame::Frame::new(&device));
        }

        Renderer {
            core,
            device,
            swapchain,

            data,

            layers,
            layer_graph,

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

        for layer in &self.layers {
            layer.record_one(&self.device, self.current_frame, self.present_index);
        }

        let mut present_wait_semaphores = Vec::<vk::Semaphore>::new();

        let mut layer_submit_infos = Vec::<LayerSubmitInfo>::with_capacity(self.layer_graph.node_count());

        let mut nodes = self.layer_graph.breadth_first_backwards("final_layer");
        nodes.reverse();

        let mut present_info_set = false;
        for node in nodes {
            let mut wait_semaphores = Vec::<vk::Semaphore>::new();
            let mut wait_stages = Vec::<vk::PipelineStageFlags>::new();
            let mut signal_semaphores = Vec::<vk::Semaphore>::new();

            let dependencies = self.layer_graph.get_prev_edges(&node.name);

            for dependency in dependencies {
                wait_semaphores.push(self.get_layer(&self.layer_graph.get_src_node(dependency).name).semaphore.semaphore);
                wait_stages.push(dependency.info.stage);
            }

            signal_semaphores.push(self.get_layer(&node.name).semaphore.semaphore);

            let mut fence = vk::Fence::null();

            let layer = self.get_layer(&node.name);
            if layer.present {
                assert!(!present_info_set, "Error: Multiple layers marked as present");
                present_info_set = true;

                wait_semaphores.push(active_frame.image_available_semaphore.semaphore);
                wait_stages.push(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT);
                fence = active_frame.in_flight_fence.fence;
                present_wait_semaphores = signal_semaphores.clone();
            }

            let command_buffers = vec![layer.commands.buffers[self.current_frame]];
            let queue = self.device.get_queue(layer.exec).0;

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

    pub unsafe fn add_buffers(&mut self, name: &str, builder: buffer::BufferBuilder) {
        self.data.add_buffers(&self.core, &self.device, name, builder);
    }

    pub unsafe fn add_images(&mut self, name: &str, builder: image::ImageBuilder) {
        self.data.add_images(&self.core, &self.device, name, builder);
    }

    pub fn get_buffers(&self, name: &str) -> &Vec<Buffer> {
        self.data.get_buffers(name)
    }

    pub fn get_images(&self, name: &str) -> &Vec<Image> {
        self.data.get_images(name)
    }

    pub unsafe fn add_layer(&mut self, name: &str, present: bool, exec: layer::LayerExecution) {
        self.layers.push(layer::Layer::new(&self.core, &self.device, self.frames_in_flight, present, exec));
        self.layer_graph.add_node(name, self.layers.len() - 1);
    }

    pub unsafe fn add_layer_dependency(&mut self, src: &str, dst: &str, stage: vk::PipelineStageFlags) {
        self.layer_graph.add_edge(src, dst, LayerDependencyInfo { stage });
    }

    pub unsafe fn add_compute_pass(&mut self, layer_name: &str, pass_name: &str, builder: compute_pass::ComputePassBuilder) {
        let pass = builder.build(&self.core, &self.device);
        self.get_layer_mut(layer_name).add_compute_pass(pass_name, pass);
    }

    pub unsafe fn add_graphics_pass<T: VertexAttributes>(&mut self, layer_name: &str, pass_name: &str, builder: graphics_pass::GraphicsPassBuilder<T>) {
        let pass = builder.build(&self.core, &self.device);
        self.get_layer_mut(layer_name).add_graphics_pass(pass_name, pass);
    }

    pub fn add_pass_dependency(&mut self, layer_name: &str, src_name: &str, dst_name: &str, dep: Option<PassDependency>) {
        self.get_layer_mut(layer_name).add_pass_dependency(src_name, dst_name, dep);
    }

    pub fn get_layer(&self, name: &str) -> &layer::Layer {
        let layer_ref = self.layer_graph.get_node(name).data;
        &self.layers[layer_ref]
    }

    pub fn get_layer_mut(&mut self, name: &str) -> &mut layer::Layer {
        let layer_ref = self.layer_graph.get_node(name).data;
        &mut self.layers[layer_ref]
    }

    pub unsafe fn fill_buffer<T>(&mut self, name: &str, data: &Vec<T>) {
        self.data.get_buffers(name)[self.current_frame].fill(&self.device, &data);
    }
}