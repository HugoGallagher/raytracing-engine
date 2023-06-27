use crate::renderer::device::Device;
use crate::renderer::graphics_pipeline::GraphicsPipeline;
use crate::renderer::image::Image2D;
use crate::renderer::framebuffer::Framebuffer;
use crate::renderer::semaphore::Semaphore;
use crate::renderer::fence::Fence;

pub struct Frame {
    pub framebuffer: Framebuffer,
    
    pub compute_finished_semaphore: Semaphore,
    pub render_finished_semaphore: Semaphore,
    pub image_available_semaphore: Semaphore,
    pub in_flight_fence: Fence,
}

impl Frame {
    pub unsafe fn new(d: &Device, g: &GraphicsPipeline, target: &Image2D) -> Frame {
        let framebuffer = Framebuffer::new(d, g, target);

        let compute_finished_semaphore = Semaphore::new(d);
        let render_finished_semaphore = Semaphore::new(d);
        let image_available_semaphore = Semaphore::new(d);

        let in_flight_fence = Fence::new(d, true);

        Frame {
            framebuffer,

            compute_finished_semaphore,
            render_finished_semaphore,
            image_available_semaphore,
            in_flight_fence,
        }
    }
}