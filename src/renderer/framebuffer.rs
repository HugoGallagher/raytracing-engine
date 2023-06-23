use ash::version::DeviceV1_0;
use ash::vk;

use crate::renderer::device::Device;
use crate::renderer::graphics_pipeline::GraphicsPipeline;
use crate::renderer::image::Image2D;

pub struct Framebuffer {
    pub framebuffers: Vec<vk::Framebuffer>
}

impl Framebuffer {
    pub unsafe fn new(d: &Device, g: &GraphicsPipeline, targets: &Vec<Image2D>) -> Framebuffer {
        let framebuffers: Vec<vk::Framebuffer> = targets.iter().map(|&i| {
            let views = [i.view];
            
            let framebuffer_ci = vk::FramebufferCreateInfo::builder()
                .render_pass(g.render_pass)
                .attachments(&views)
                .width(d.surface_extent.width)
                .height(d.surface_extent.height)
                .layers(1);

            d.device.create_framebuffer(&framebuffer_ci, None).unwrap()
        }).collect();

        Framebuffer {
            framebuffers: framebuffers,
        }
    }
}