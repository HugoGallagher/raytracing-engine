mod core;
mod device;
mod swapchain;
mod buffer;
mod image;
mod sampler;
mod graphics_pipeline;
mod compute_pipeline;
mod descriptors;
mod shader;
mod framebuffer;
mod commands;
mod fence;
mod semaphore;
mod frame;

use std::ffi::c_void;

use ash::{vk, version::DeviceV1_0};

use crate::{window::Window, math::vec::Vec4, renderer::descriptors::storage_descriptor};

pub struct Renderer {
    core: core::Core,
    device: device::Device,
    swapchain: swapchain::Swapchain,
    descriptors: descriptors::Descriptors,
    compute_pipeline: compute_pipeline::ComputePipeline,
    graphics_pipeline: graphics_pipeline::GraphicsPipeline,
    framebuffer: framebuffer::Framebuffer,
    commands: commands::Commands,
    frames: Vec<frame::Frame>,

    frames_in_flight: u32,
    current_frame: u32,
}

impl Renderer {
    pub unsafe fn new(w: &Window) -> Renderer {
        const FRAMES_IN_FLIGHT: u32 = 2;

        let core = core::Core::new(true, w);
        let device = device::Device::new(&core, w);
        let swapchain = swapchain::Swapchain::new(&core, &device);

        let uniform_builder = descriptors::uniform_descriptor::UniformDescriptorBuilder::new()
            .buffer_count(FRAMES_IN_FLIGHT as usize)
            .buffer_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .buffer_size(128);

        let storage_builder = descriptors::storage_descriptor::StorageDescriptorBuilder::new()
            .buffer_count(FRAMES_IN_FLIGHT as usize)
            .buffer_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .buffer_size(128);

        let descriptors = descriptors::DescriptorsBuilder::new()
            .count(FRAMES_IN_FLIGHT as usize)
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .add_uniform_builder(uniform_builder)
            .add_storage_builder(storage_builder)
            .build(&core, &device);

        let compute_pipeline = compute_pipeline::ComputePipeline::new(&device, "test.comp");
        let graphics_pipeline = graphics_pipeline::GraphicsPipeline::new(&core, &device, &swapchain, &descriptors, "draw_to_screen.vert", "draw_to_screen.frag");
        let framebuffer = framebuffer::Framebuffer::new(&device, &graphics_pipeline, &swapchain.images);
        let commands = commands::Commands::new(&device, &swapchain, device.queue_graphics.1, FRAMES_IN_FLIGHT);

        let mut frames = Vec::<frame::Frame>::new();

        for _ in  0..FRAMES_IN_FLIGHT {
            frames.push(frame::Frame::new(&device));
        }

        Renderer {
            core,
            device,
            swapchain,
            descriptors,
            compute_pipeline,
            graphics_pipeline,
            framebuffer,
            commands,
            frames,

            frames_in_flight: FRAMES_IN_FLIGHT,
            current_frame: 0,
        }
    }

    pub unsafe fn draw(&mut self) {
        self.current_frame = (self.current_frame + 1) % self.frames_in_flight;

        self.device.device.wait_for_fences(&[self.frames[self.current_frame as usize].in_flight_fence.fence], true, u64::MAX).unwrap();
        self.device.device.reset_fences(&[self.frames[self.current_frame as usize].in_flight_fence.fence]).unwrap();

        let present_index = self.swapchain.swapchain_init.acquire_next_image(self.swapchain.swapchain, u64::MAX, self.frames[self.current_frame as usize].image_available_semaphore.semaphore, vk::Fence::null()).unwrap().0 as usize;
        let present_indices = [present_index as u32];

        let draw_col = Vec4::new(1.0, 0.0, 1.0, 1.0);
        let draw_col_ptr: *const Vec4 = &draw_col;

        let draw_cols = vec![Vec4::new(1.0, 0.0, 0.0, 1.0), Vec4::new(0.0, 1.0, 0.0, 1.0), Vec4::new(0.0, 0.0, 1.0, 1.0)];
        let draw_cols_ptr: *const Vec4 = draw_cols.as_ptr();

        self.descriptors.uniforms[0].buffers[self.current_frame as usize].fill(&self.device, draw_col_ptr as *const c_void, 16);
        self.descriptors.ssbos[0].buffers[self.current_frame as usize].fill(&self.device, draw_cols_ptr as *const c_void, 48);

        self.commands.record_one(&self.device, self.current_frame as usize, |b| {
            let clear_values = [vk::ClearValue { color: vk::ClearColorValue { float32: [0.5, 0.2, 0.5, 0.0]}}];

            let rect = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.device.surface_extent,
            };

            let render_pass_bi = vk::RenderPassBeginInfo::builder()
                .render_pass(self.graphics_pipeline.render_pass)
                .framebuffer(self.framebuffer.framebuffers[present_index])
                .render_area(rect)
                .clear_values(&clear_values);

            self.descriptors.bind(&self.device, &b, vk::PipelineBindPoint::GRAPHICS, &self.graphics_pipeline.pipeline_layout, self.current_frame as usize);

            self.device.device.cmd_begin_render_pass(b, &render_pass_bi, vk::SubpassContents::INLINE);

            self.device.device.cmd_bind_pipeline(b, vk::PipelineBindPoint::GRAPHICS, self.graphics_pipeline.pipeline);
            self.device.device.cmd_set_viewport(b, 0, &[self.graphics_pipeline.viewport]);
            self.device.device.cmd_set_scissor(b, 0, &[self.graphics_pipeline.scissor]);

            self.device.device.cmd_draw(b, 6, 1, 0, 0);
   
            self.device.device.cmd_end_render_pass(b);
        });

        let wait_semaphores = [self.frames[self.current_frame as usize].image_available_semaphore.semaphore];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        let signal_semaphores = [self.frames[self.current_frame as usize].render_finished_semaphore.semaphore];

        let command_buffers = [self.commands.buffers[self.current_frame as usize]];

        let submit_i = vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .signal_semaphores(&signal_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .build();

        self.device.device.queue_submit(self.device.queue_graphics.0, &[submit_i], self.frames[self.current_frame as usize].in_flight_fence.fence).unwrap();

        let swapchains = [self.swapchain.swapchain];

        let present_i = vk::PresentInfoKHR::builder()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&present_indices);

        self.swapchain.swapchain_init.queue_present(self.device.queue_present.0, &present_i).unwrap();
    }
}