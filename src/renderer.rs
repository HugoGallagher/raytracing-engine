mod core;
mod device;
mod image;
mod swapchain;
mod graphics_pipeline;
mod shader;

use crate::window::Window;

pub struct Renderer {
    core: core::Core,
    device: device::Device,
    swapchain: swapchain::Swapchain,
    graphics_pipeline: graphics_pipeline::GraphicsPipeline,
}

impl Renderer {
    pub unsafe fn new(w: &Window) -> Renderer {
        let core = core::Core::new(true, w);
        let device = device::Device::new(&core, w);
        let swapchain = swapchain::Swapchain::new(&core, &device);
        let graphics_pipeline = graphics_pipeline::GraphicsPipeline::new(&core, &device, &swapchain, "vert.vert", "frag.frag");

        Renderer {
            core,
            device,
            swapchain,
            graphics_pipeline,
        }
    }
}