use std::{ffi::c_void, mem};

use ash::vk;

use crate::{renderer::core::Core, math::vec::Vec4};
use crate::renderer::device::Device;
use crate::renderer::buffer::{Buffer, BufferBuilder};

pub struct VertexAttribute {
    pub format: vk::Format,
    pub offset: usize,
}

pub trait VertexAttributes {
    fn get_attribute_data() -> Vec<VertexAttribute>;
}

pub struct NoVertices {}

impl VertexAttributes for NoVertices {
    fn get_attribute_data() -> Vec<VertexAttribute> {
        vec![]
    }
}

impl VertexAttributes for Vec4 {
    fn get_attribute_data() -> Vec<VertexAttribute> {
        vec![
            VertexAttribute { format: vk::Format::R32G32B32A32_SFLOAT, offset: 0 },
        ]
    }
}

pub struct VertexBuffer {
    pub binding_desc: vk::VertexInputBindingDescription,
    pub attrib_descs: Vec<vk::VertexInputAttributeDescription>,
    pub buffer: Buffer,
    pub index_buffer: Option<Buffer>,
}

impl VertexBuffer {
    pub unsafe fn new<T: VertexAttributes>(c: &Core, d: &Device, verts: &Vec<T>, indices: Option<&Vec<u32>>) -> VertexBuffer {
        let binding_desc = vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(std::mem::size_of::<T>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build();

        let vertex_attribs = T::get_attribute_data();
        let mut attrib_descs: Vec<vk::VertexInputAttributeDescription> = Vec::with_capacity(vertex_attribs.len());

        for (i, a) in vertex_attribs.iter().enumerate() {
            attrib_descs.push(vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(i as u32)
                .format(a.format)
                .offset(a.offset as u32)
                .build());
        }

        let buffer = BufferBuilder::new()
            .size(mem::size_of::<T>() * verts.len())
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .properties(vk::MemoryPropertyFlags::DEVICE_LOCAL)
            .build_with_data(c, d, verts.as_ptr() as *const c_void);

        let index_buffer = match indices {
            Some(is) => {
                Some(BufferBuilder::new()
                    .size(mem::size_of::<u32>() * is.len())
                    .usage(vk::BufferUsageFlags::INDEX_BUFFER)
                    .sharing_mode(vk::SharingMode::EXCLUSIVE)
                    .properties(vk::MemoryPropertyFlags::DEVICE_LOCAL)
                    .build_with_data(c, d, is.as_ptr() as *const c_void))
            },
            None => None,
        };

        VertexBuffer {
            binding_desc,
            attrib_descs,
            buffer,
            index_buffer,
        }
    }
}