mod core;
mod device;
mod swapchain;
mod buffer;
mod image;
mod sampler;
mod descriptors;
mod shader;
mod framebuffer;
mod commands;
mod compute_pipeline;
mod graphics_pipeline;
mod compute_pass;
mod graphics_pass;
mod compute_layer;
mod fence;
mod semaphore;
mod frame;
mod mesh;
mod push_constant;

use std::{mem, ffi::c_void};

use ash::{vk, version::DeviceV1_0};

use crate::{window::Window, math::{vec::{Vec4, Vec3}, mat::Mat4}, renderer::{descriptors::{storage_descriptor, image_descriptor, sampler_descriptor, uniform_descriptor}, mesh::Tri}};

#[repr(C)]
pub struct PushConstantData {
    pub view: Mat4,
    pub pos: Vec3,
    pub downscale: u32,
    pub tri_count: u32,
}

pub struct Renderer {
    core: core::Core,
    device: device::Device,
    swapchain: swapchain::Swapchain,

    pub push_constant: PushConstantData,
    pub compute_layer: compute_layer::ComputeLayer,
    
    graphics_descriptors: descriptors::Descriptors,
    graphics_pipeline: graphics_pipeline::GraphicsPipeline,
    graphics_commands: commands::Commands,

    frames: Vec<frame::Frame>,

    tris: Vec<Tri>,

    frames_in_flight: usize,
    current_frame: usize,
}

impl Renderer {
    pub unsafe fn new(w: &Window) -> Renderer {
        const FRAMES_IN_FLIGHT: u32 = 2;

        const MAX_TRIS: usize = 8192;

        let core = core::Core::new(true, w);
        let device = device::Device::new(&core, w);
        let swapchain = swapchain::Swapchain::new(&core, &device);

        let mut compute_layer = compute_layer::ComputeLayer::new(&core, &device, FRAMES_IN_FLIGHT as usize);

        let buffer_builder = buffer::BufferBuilder::new()
            .size(mem::size_of::<Tri>() * MAX_TRIS)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .usage(vk::BufferUsageFlags::STORAGE_BUFFER);

        let image_builder = image::Image2DBuilder::new()
            .width(1280)
            .height(720)
            .usage(vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED)
            .format(vk::Format::R8G8B8A8_UNORM);
        
        compute_layer.add_buffer(&core, &device, "tris", buffer_builder);
        compute_layer.add_image(&core, &device, "output", image_builder);

        let image_descriptor_builder = descriptors::image_descriptor::ImageDescriptorBuilder::new()
            .images(&compute_layer.images.get("output").unwrap());
            
        let sampler_descriptor_builder = descriptors::sampler_descriptor::SamplerDescriptorBuilder::new()
            .images(&compute_layer.images.get("output").unwrap());

        let storage_descriptor_builder = descriptors::storage_descriptor::StorageDescriptorBuilder::new()
            .buffers(&compute_layer.buffers.get("tris").unwrap());

        let compute_descriptors_builder = descriptors::DescriptorsBuilder::new()
            .count(FRAMES_IN_FLIGHT as usize)
            .stage(vk::ShaderStageFlags::COMPUTE)
            .add_storage_builder(storage_descriptor_builder)
            .add_image_builder(image_descriptor_builder);

        let mut tris = Vec::<Tri>::with_capacity(MAX_TRIS);
        
        mesh::parse_obj(&mut tris, "res/meshes/asdf.obj");

        let push_constant = PushConstantData {
            view: Mat4::identity(),
            pos: Vec3::zero(),
            downscale: 1,
            tri_count: tris.len() as u32,
        };

        compute_layer.add_pass(&core, &device, Some(compute_descriptors_builder), Some(mem::size_of::<PushConstantData>()), "raytracer.comp", (1280 / 16, 720 / 16, 1));

        let graphics_descriptors = descriptors::DescriptorsBuilder::new()
            .count(FRAMES_IN_FLIGHT as usize)
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .add_sampler_builder(sampler_descriptor_builder)
            .build(&core, &device);

        let graphics_pipeline = graphics_pipeline::GraphicsPipeline::new(&core, &device, &swapchain, &graphics_descriptors, "draw_to_screen.vert", "draw_to_screen.frag");
        let graphics_commands = commands::Commands::new(&device, device.queue_graphics.1, FRAMES_IN_FLIGHT as usize);

        let mut frames = Vec::<frame::Frame>::new();

        for i in 0..FRAMES_IN_FLIGHT {
            frames.push(frame::Frame::new(&device, &graphics_pipeline, &swapchain.images[i as usize]));
        }

        Renderer {
            core,
            device,
            swapchain,

            push_constant,
            compute_layer,

            graphics_descriptors,
            graphics_pipeline,
            graphics_commands,

            frames,
            tris,

            frames_in_flight: FRAMES_IN_FLIGHT as usize,
            current_frame: 0,
        }
    }

    pub unsafe fn draw(&mut self) {
        self.current_frame = (self.current_frame + 1) % self.frames_in_flight;

        let active_frame = &self.frames[self.current_frame];

        self.device.device.wait_for_fences(&[active_frame.in_flight_fence.fence], true, u64::MAX).unwrap();
        self.device.device.reset_fences(&[active_frame.in_flight_fence.fence]).unwrap();

        let present_index = self.swapchain.swapchain_init.acquire_next_image(self.swapchain.swapchain, u64::MAX, active_frame.image_available_semaphore.semaphore, vk::Fence::null()).unwrap().0 as usize;
        let present_indices = [present_index as u32];
        
        self.compute_layer.fill_push_constant(0, &self.push_constant);
        self.compute_layer.buffers.get("tris").unwrap()[self.current_frame].fill(&self.device, self.tris.as_ptr() as *const c_void, self.tris.len() * mem::size_of::<Tri>());

        self.compute_layer.record_one(&self.device, self.current_frame);

        self.graphics_commands.record_one(&self.device, self.current_frame as usize, |b| {
            let clear_values = [vk::ClearValue { color: vk::ClearColorValue { float32: [0.0, 0.0, 0.0, 0.0]}}];

            let rect = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.device.surface_extent,
            };

            let render_pass_bi = vk::RenderPassBeginInfo::builder()
                .render_pass(self.graphics_pipeline.render_pass)
                .framebuffer(self.frames[present_index].framebuffer.framebuffer)
                .render_area(rect)
                .clear_values(&clear_values);

            self.graphics_descriptors.bind(&self.device, &b, vk::PipelineBindPoint::GRAPHICS, &self.graphics_pipeline.pipeline_layout, self.current_frame);

            self.device.device.cmd_begin_render_pass(b, &render_pass_bi, vk::SubpassContents::INLINE);

            self.device.device.cmd_bind_pipeline(b, vk::PipelineBindPoint::GRAPHICS, self.graphics_pipeline.pipeline);
            self.device.device.cmd_set_viewport(b, 0, &[self.graphics_pipeline.viewport]);
            self.device.device.cmd_set_scissor(b, 0, &[self.graphics_pipeline.scissor]);

            self.device.device.cmd_draw(b, 6, 1, 0, 0);
   
            self.device.device.cmd_end_render_pass(b);
        });

        let compute_wait_semaphores = [active_frame.image_available_semaphore.semaphore];
        let compute_signal_semaphores = [active_frame.compute_finished_semaphore.semaphore];
        let compute_wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        let graphics_wait_semaphores = [active_frame.image_available_semaphore.semaphore, active_frame.compute_finished_semaphore.semaphore];
        let graphics_signal_semaphores = [active_frame.render_finished_semaphore.semaphore];
        let graphics_wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT, vk::PipelineStageFlags::COMPUTE_SHADER];

        let compute_command_buffers = [self.compute_layer.commands.buffers[self.current_frame]];
        let graphics_command_buffers = [self.graphics_commands.buffers[self.current_frame]];

        let compute_submit_i = vk::SubmitInfo {
            wait_semaphore_count: 0,
            p_wait_semaphores: compute_wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: compute_wait_stages.as_ptr(),
            signal_semaphore_count: 1,
            p_signal_semaphores: compute_signal_semaphores.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: compute_command_buffers.as_ptr(),
            ..Default::default()
        };

        let graphics_submit_i = vk::SubmitInfo {
            wait_semaphore_count: 2,
            p_wait_semaphores: graphics_wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: graphics_wait_stages.as_ptr(),
            signal_semaphore_count: 1,
            p_signal_semaphores: graphics_signal_semaphores.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: graphics_command_buffers.as_ptr(),
            ..Default::default()
        };

        self.device.device.queue_submit(self.device.queue_compute.0, &[compute_submit_i], vk::Fence::null()).unwrap();
        self.device.device.queue_submit(self.device.queue_graphics.0, &[graphics_submit_i], active_frame.in_flight_fence.fence).unwrap();

        let swapchains = [self.swapchain.swapchain];

        let present_i = vk::PresentInfoKHR::builder()
            .wait_semaphores(&graphics_signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&present_indices);

        self.swapchain.swapchain_init.queue_present(self.device.queue_present.0, &present_i).unwrap();
    }
}