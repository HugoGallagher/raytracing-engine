use ash::{version::DeviceV1_0, vk};

use crate::renderer::device::Device;
use crate::renderer::swapchain::Swapchain;

pub struct Commands {
    pub pool: vk::CommandPool,
    pub buffers: Vec<vk::CommandBuffer>,
}

impl Commands {
    pub unsafe fn new(d: &Device, s: &Swapchain, q: u32, c: u32) -> Commands {
        let pool_ci = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(q);

        let pool = d.device.create_command_pool(&pool_ci, None).unwrap();

        let buffer_alloc_i = vk::CommandBufferAllocateInfo::builder()
            .command_pool(pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(c);

        let buffers = d.device.allocate_command_buffers(&buffer_alloc_i).unwrap();

        Commands {
            pool: pool,
            buffers: buffers,
        }
    }

    pub unsafe fn record_all<F: Fn(usize, vk::CommandBuffer)>(&self, d: &Device, r: F) {
        for i in 0..self.buffers.len() {
            self.record_one(d, i, |b| { r(i, b) });

            /*
            d.device.reset_command_buffer(self.buffers[i], vk::CommandBufferResetFlags::RELEASE_RESOURCES).unwrap();

            let buffer_bi = vk::CommandBufferBeginInfo::builder();

            d.device.begin_command_buffer(self.buffers[i], &buffer_bi).unwrap();
            
            r(i, self.buffers[i]);

            d.device.end_command_buffer(self.buffers[i]).unwrap();
            */
        }
    }

    pub unsafe fn record_one<F: Fn(vk::CommandBuffer)>(&self, d: &Device, i: usize, r: F) {
        d.device.reset_command_buffer(self.buffers[i], vk::CommandBufferResetFlags::RELEASE_RESOURCES).unwrap();

        let buffer_bi = vk::CommandBufferBeginInfo::builder();

        d.device.begin_command_buffer(self.buffers[i], &buffer_bi).unwrap();

        r(self.buffers[i]);

        d.device.end_command_buffer(self.buffers[i]).unwrap();
    }
}