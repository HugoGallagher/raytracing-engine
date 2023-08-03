use ash::vk;

use crate::renderer::device::Device;
use crate::renderer::graphics_pipeline::GraphicsPipeline;
use crate::renderer::image::Image;

pub struct Framebuffer {
    pub framebuffer: vk::Framebuffer
}

impl Framebuffer {
    pub unsafe fn new(d: &Device, g: &GraphicsPipeline, target: &Image, extent: Option<vk::Extent2D>) -> Framebuffer {
        let mut views = vec![target.view];

        if let Some(depth_image) = g.depth_image {
            views.push(depth_image.view);
        }

        let (width, height) = match extent {
            Some(e) => (e.width, e.height),
            None => (target.width, target.height),
        };
            
        let framebuffer_ci = vk::FramebufferCreateInfo::builder()
            .render_pass(g.render_pass)
            .attachments(&views)
            .width(width)
            .height(height)
            .layers(1)
            .build();

        let framebuffer = d.device.create_framebuffer(&framebuffer_ci, None).unwrap();

        Framebuffer {
            framebuffer,
        }
    }

    pub unsafe fn new_many(d: &Device, g: &GraphicsPipeline, targets: &Vec<Image>, extent: Option<vk::Extent2D>) -> Vec<Framebuffer> {
        let mut framebuffers = Vec::<Framebuffer>::new();

        for target in targets {
            framebuffers.push(Framebuffer::new(d, g, target, extent));
        }

        framebuffers
    }
}