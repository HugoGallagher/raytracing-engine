mod core;
mod device;
mod image;
mod swapchain;
mod graphics_pipeline;
mod shader;
mod framebuffer;
mod commands;
mod fence;
mod semaphore;
mod frame;

use ash::{vk, version::DeviceV1_0};

use crate::window::Window;

pub struct Renderer {
    core: core::Core,
    device: device::Device,
    swapchain: swapchain::Swapchain,
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
        let graphics_pipeline = graphics_pipeline::GraphicsPipeline::new(&core, &device, &swapchain, "vert.vert", "frag.frag");
        let framebuffer = framebuffer::Framebuffer::new(&device, &swapchain, &graphics_pipeline, &swapchain.images);
        let commands = commands::Commands::new(&device, &swapchain, device.queue_graphics.1, FRAMES_IN_FLIGHT);

        let mut frames = Vec::<frame::Frame>::new();

        for _ in  0..FRAMES_IN_FLIGHT {
            frames.push(frame::Frame::new(&device));
        }

        Renderer {
            core,
            device,
            swapchain,
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

            self.device.device.cmd_begin_render_pass(b, &render_pass_bi, vk::SubpassContents::INLINE);

            self.device.device.cmd_bind_pipeline(b, vk::PipelineBindPoint::GRAPHICS, self.graphics_pipeline.pipeline);
            self.device.device.cmd_set_viewport(b, 0, &[self.graphics_pipeline.viewport]);
            self.device.device.cmd_set_scissor(b, 0, &[self.graphics_pipeline.scissor]);

            self.device.device.cmd_draw(b, 3, 1, 0, 0);

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