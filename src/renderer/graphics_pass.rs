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

#[derive(Copy, Clone)]
pub struct GraphicsPassDrawInfo {
    pub vertex_count: u32,
    pub index_count: u32,
    pub instance_count: u32,
    pub first_vertex: u32,
    pub first_instance: u32,
    pub vertex_offset: i32,
}

pub struct GraphicsPassBuilder<'a, T: VertexAttributes> {
    draw_info: Option<GraphicsPassDrawInfo>,
    targets: Option<&'a Vec<Image>>,
    extent: Option<vk::Extent2D>,
    offset: Option<vk::Offset2D>,
    vs: Option<&'a str>,
    fs: Option<&'a str>,
    verts: Option<&'a Vec<T>>,
    vertex_indices: Option<&'a Vec<u32>>,
    push_constant_builder: Option<PushConstantBuilder>,
    descriptors_builder: Option<DescriptorsBuilder>,
    with_depth_buffer: bool,
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

impl <'a, T: VertexAttributes> GraphicsPassBuilder<'a, T> {
    pub fn new() -> GraphicsPassBuilder<'a, T> {
        GraphicsPassBuilder {
            draw_info: None,
            targets: None,
            extent: None,
            offset: None,
            vs: None,
            fs: None,
            verts: None,
            vertex_indices: None,
            push_constant_builder: None,
            descriptors_builder: None,
            with_depth_buffer: false,
        }
    }

    pub fn draw_info(mut self, draw_info: GraphicsPassDrawInfo) -> GraphicsPassBuilder<'a, T> {
        self.draw_info = Some(draw_info);

        self
    }

    pub fn targets(mut self, targets: &'a Vec<Image>) -> GraphicsPassBuilder<'a, T> {
        self.targets = Some(targets);

        self
    }

    pub fn extent(mut self, extent: vk::Extent2D) -> GraphicsPassBuilder<'a, T> {
        self.extent = Some(extent);

        self
    }

    pub fn offset(mut self, offset: vk::Offset2D) -> GraphicsPassBuilder<'a, T> {
        self.offset = Some(offset);

        self
    }

    pub fn vertex_shader(mut self, vs: &'a str) -> GraphicsPassBuilder<'a, T> {
        self.vs = Some(vs);

        self
    }

    pub fn fragment_shader(mut self, fs: &'a str) -> GraphicsPassBuilder<'a, T> {
        self.fs = Some(fs);

        self
    }

    pub fn verts(mut self, verts: &'a Vec<T>) -> GraphicsPassBuilder<'a, T> {
        self.verts = Some(verts);

        self
    }

    pub fn vertex_indices(mut self, vertex_indices: &'a Vec<u32>) -> GraphicsPassBuilder<'a, T> {
        self.vertex_indices = Some(vertex_indices);

        self
    }

    pub fn push_constant_builder(mut self, push_constant_builder: PushConstantBuilder) -> GraphicsPassBuilder<'a, T> {
        self.push_constant_builder = Some(push_constant_builder);

        self
    }

    pub fn descriptors_builder(mut self, descriptors_builder: DescriptorsBuilder) -> GraphicsPassBuilder<'a, T> {
        self.descriptors_builder = Some(descriptors_builder);

        self
    }

    pub fn with_depth_buffer(mut self) -> GraphicsPassBuilder<'a, T> {
        self.with_depth_buffer = true;

        self
    }

    pub unsafe fn build(self, c: &Core, d: &Device) -> GraphicsPass {
        GraphicsPass::new(c, d, self.targets.expect("Error: Graphics pass builder has no targets"), self.extent, self.offset, self.verts, self.vertex_indices, self.descriptors_builder, self.push_constant_builder, self.vs.expect("Error: Graphics pass builder has no vertex shader"), self.fs.expect("Error: Graphics pass builder has no fragment shader"), self.with_depth_buffer, self.draw_info.expect("Error: Graphics pass builder has no draw info"))
    }
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