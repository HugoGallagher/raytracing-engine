use ash::vk;

use crate::renderer::core::Core;
use crate::renderer::device::Device;
use crate::renderer::descriptors::{Descriptors, DescriptorsBuilder};
use crate::renderer::vertex_buffer::{VertexBuffer, VertexAttributes};
use crate::renderer::push_constant::PushConstantBuilder;
use crate::renderer::graphics_pipeline::GraphicsPipeline;
use crate::renderer::framebuffer::Framebuffer;
use crate::renderer::push_constant::PushConstant;
use crate::renderer::image::Image2D;

pub struct GraphicsPass {
    pub push_constant: Option<PushConstant>,
    pub descriptors: Option<Descriptors>,
    pub vertex_buffer: Option<VertexBuffer>,
    pub pipeline: GraphicsPipeline,
    pub framebuffers: Vec<Framebuffer>,
}

impl GraphicsPass {
    pub unsafe fn new<T: VertexAttributes>(c: &Core, d: &Device, targets: &Vec<Image2D>, verts: Option<&Vec<T>>, descriptors_builder: Option<DescriptorsBuilder>, push_constant_builder: Option<PushConstantBuilder>, vs: &str, fs: &str) -> GraphicsPass {
        let descriptors = match descriptors_builder {
            Some(de_b) => Some(de_b.build(c, d)),
            None => None
        };

        let descriptor_set_layout = match descriptors.as_ref() {
            Some(de) => Some(de.set_layout),
            None => None
        };

        let push_constant = match push_constant_builder {
            Some(builder) => Some(builder.build()),
            None => None
        };

        let vertex_buffer = match verts {
            Some(v) => Some(VertexBuffer::new(c, d, v)),
            None => None
        };

        let target_extent = (targets[0].width, targets[0].height);
        
        let pipeline = GraphicsPipeline::new(c, d, target_extent, vertex_buffer.as_ref(), descriptor_set_layout, push_constant.as_ref(), vs, fs);

        let framebuffers = Framebuffer::new_many(d, &pipeline, targets);

        GraphicsPass {
            push_constant,
            descriptors,
            vertex_buffer,
            pipeline,
            framebuffers,
        }
    }
}