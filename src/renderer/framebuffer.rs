use ash::vk;

use crate::renderer::device::Device;
use crate::renderer::graphics_pipeline::GraphicsPipeline;
use crate::renderer::image::Image2D;

pub struct Framebuffer {
    pub framebuffer: vk::Framebuffer
}

impl Framebuffer {
    pub unsafe fn new(d: &Device, g: &GraphicsPipeline, target: &Image2D) -> Framebuffer {
        let views = [target.view];
            
        let framebuffer_ci = vk::FramebufferCreateInfo::builder()
            .render_pass(g.render_pass)
            .attachments(&views)
            .width(d.surface_extent.width)
            .height(d.surface_extent.height)
            .layers(1);

        let framebuffer = d.device.create_framebuffer(&framebuffer_ci, None).unwrap();

        Framebuffer {
            framebuffer,
        }
    }
}