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

use crate::{window::Window, math::vec::Vec4, renderer::descriptors::{storage_descriptor, image_descriptor, sampler_descriptor, uniform_descriptor}};

pub struct Renderer {
    core: core::Core,
    device: device::Device,
    swapchain: swapchain::Swapchain,

    images: Vec<image::Image2D>,

    compute_descriptors: descriptors::Descriptors,
    compute_pipeline: compute_pipeline::ComputePipeline,
    compute_commands: commands::Commands,
    
    graphics_descriptors: descriptors::Descriptors,
    graphics_pipeline: graphics_pipeline::GraphicsPipeline,
    graphics_commands: commands::Commands,

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

        let images = image::Image2DBuilder::new()
            .width(500)
            .height(500)
            .usage(vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED)
            .format(vk::Format::R8G8B8A8_UNORM)
            .build_many(&core, &device, FRAMES_IN_FLIGHT);

        let image_descriptor_builder = descriptors::image_descriptor::ImageDescriptorBuilder::new()
            .images(&images);
            
        let sampler_descriptor_builder = descriptors::sampler_descriptor::SamplerDescriptorBuilder::new()
            .images(&images);

        let compute_descriptors = descriptors::DescriptorsBuilder::new()
            .count(FRAMES_IN_FLIGHT as usize)
            .stage(vk::ShaderStageFlags::COMPUTE)
            .add_image_builder(image_descriptor_builder)
            .build(&core, &device);

        let graphics_descriptors = descriptors::DescriptorsBuilder::new()
            .count(FRAMES_IN_FLIGHT as usize)
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .add_sampler_builder(sampler_descriptor_builder)
            .build(&core, &device);

        let compute_pipeline = compute_pipeline::ComputePipeline::new(&device, &compute_descriptors, "test.comp");
        let graphics_pipeline = graphics_pipeline::GraphicsPipeline::new(&core, &device, &swapchain, &graphics_descriptors, "draw_to_screen.vert", "draw_to_screen.frag");

        let compute_commands = commands::Commands::new(&device, device.queue_compute.1, FRAMES_IN_FLIGHT);
        let graphics_commands = commands::Commands::new(&device, device.queue_graphics.1, FRAMES_IN_FLIGHT);

        let mut frames = Vec::<frame::Frame>::new();

        for i in 0..FRAMES_IN_FLIGHT {
            frames.push(frame::Frame::new(&device, &graphics_pipeline, &swapchain.images[i as usize]));
        }

        Renderer {
            core,
            device,
            swapchain,

            images,

            compute_descriptors,
            compute_pipeline,
            compute_commands,

            graphics_descriptors,
            graphics_pipeline,
            graphics_commands,

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

        self.compute_commands.record_one(&self.device, self.current_frame as usize, |b| {
            self.device.device.cmd_bind_pipeline(b, vk::PipelineBindPoint::COMPUTE, self.compute_pipeline.pipeline);

            self.compute_descriptors.bind(&self.device, &b, vk::PipelineBindPoint::COMPUTE, &self.compute_pipeline.pipeline_layout, self.current_frame as usize);

            self.device.device.cmd_dispatch(b, self.device.surface_extent.width / 16, self.device.surface_extent.height / 16, 1);
        });

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

            self.graphics_descriptors.bind(&self.device, &b, vk::PipelineBindPoint::GRAPHICS, &self.graphics_pipeline.pipeline_layout, self.current_frame as usize);

            self.device.device.cmd_begin_render_pass(b, &render_pass_bi, vk::SubpassContents::INLINE);

            self.device.device.cmd_bind_pipeline(b, vk::PipelineBindPoint::GRAPHICS, self.graphics_pipeline.pipeline);
            self.device.device.cmd_set_viewport(b, 0, &[self.graphics_pipeline.viewport]);
            self.device.device.cmd_set_scissor(b, 0, &[self.graphics_pipeline.scissor]);

            self.device.device.cmd_draw(b, 6, 1, 0, 0);
   
            self.device.device.cmd_end_render_pass(b);
        });

        let compute_wait_semaphores = [self.frames[self.current_frame as usize].image_available_semaphore.semaphore];
        let compute_signal_semaphores = [self.frames[self.current_frame as usize].compute_finished_semaphore.semaphore];
        let compute_wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        let graphics_wait_semaphores = [self.frames[self.current_frame as usize].compute_finished_semaphore.semaphore];
        let graphics_signal_semaphores = [self.frames[self.current_frame as usize].render_finished_semaphore.semaphore];
        let graphics_wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        let compute_command_buffers = [self.compute_commands.buffers[self.current_frame as usize]];
        let graphics_command_buffers = [self.graphics_commands.buffers[self.current_frame as usize]];

        let compute_submit_i = vk::SubmitInfo::builder()
            .wait_semaphores(&compute_wait_semaphores)
            .signal_semaphores(&compute_signal_semaphores)
            .wait_dst_stage_mask(&compute_wait_stages)
            .command_buffers(&compute_command_buffers)
            .build();

        let graphics_submit_i = vk::SubmitInfo::builder()
            .wait_semaphores(&graphics_wait_semaphores)
            .signal_semaphores(&graphics_signal_semaphores)
            .wait_dst_stage_mask(&graphics_wait_stages)
            .command_buffers(&graphics_command_buffers)
            .build();

        self.device.device.queue_submit(self.device.queue_compute.0, &[compute_submit_i], vk::Fence::null()).unwrap();
        self.device.device.queue_submit(self.device.queue_graphics.0, &[graphics_submit_i], self.frames[self.current_frame as usize].in_flight_fence.fence).unwrap();

        let swapchains = [self.swapchain.swapchain];

        let present_i = vk::PresentInfoKHR::builder()
            .wait_semaphores(&graphics_signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&present_indices);

        self.swapchain.swapchain_init.queue_present(self.device.queue_present.0, &present_i).unwrap();
    }
}