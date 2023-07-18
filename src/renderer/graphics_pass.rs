use ash::vk;

use crate::renderer::core::Core;
use crate::renderer::device::Device;
use crate::renderer::descriptors::{Descriptors, DescriptorsBuilder};
use crate::renderer::vertex_buffer::{VertexBuffer, VertexAttributes};
use crate::renderer::push_constant::PushConstantBuilder;
use crate::renderer::graphics_pipeline::GraphicsPipeline;
use crate::renderer::framebuffer::Framebuffer;
use crate::renderer::push_constant::PushConstant;
use crate::renderer::image::Image;

pub struct GraphicsPassDrawInfo {
    pub vertex_count: u32,
    pub index_count: u32,
    pub instance_count: u32,
    pub first_vertex: u32,
    pub first_instance: u32,
    pub vertex_offset: i32,
}

pub struct GraphicsPass {
    pub push_constant: Option<PushConstant>,
    pub descriptors: Option<Descriptors>,
    pub vertex_buffer: Option<VertexBuffer>,
    pub pipeline: GraphicsPipeline,
    pub framebuffers: Vec<Framebuffer>,
    pub draw_info: GraphicsPassDrawInfo,
    pub indexed: bool,
}

impl GraphicsPass {
    pub unsafe fn new<T: VertexAttributes>(c: &Core, d: &Device, targets: &Vec<Image>, extent: Option<vk::Extent2D>, offset: Option<vk::Offset2D>, verts: Option<&Vec<T>>, indices: Option<&Vec<u32>>, descriptors_builder: Option<DescriptorsBuilder>, push_constant_builder: Option<PushConstantBuilder>, vs: &str, fs: &str, with_depth_buffer: bool, draw_info: GraphicsPassDrawInfo) -> GraphicsPass {
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
            Some(v) => Some(VertexBuffer::new(c, d, v, indices)),
            None => None
        };

        let target_extent = match extent {
            Some(e) => e,
            None => vk::Extent2D { width: targets[0].width, height: targets[0].height },
        };

        let offset = match offset {
            Some(o) => o,
            None => vk::Offset2D { x: 0, y: 0 },
        };

        let target_rect = vk::Rect2D { extent: target_extent, offset };
        
        let pipeline = GraphicsPipeline::new(c, d, target_rect, vertex_buffer.as_ref(), descriptor_set_layout, push_constant.as_ref(), vs, fs, with_depth_buffer);

        let framebuffers = Framebuffer::new_many(d, &pipeline, targets, extent);

        let indexed = indices.is_some();

        GraphicsPass {
            push_constant,
            descriptors,
            vertex_buffer,
            pipeline,
            framebuffers,
            draw_info,
            indexed,
        }
    }
}