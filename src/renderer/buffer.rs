use std::ffi::c_void;

use ash::version::{InstanceV1_0, DeviceV1_0};
use ash::vk;

use crate::math::vec::Vec2;
use crate::renderer::core::Core;
use crate::renderer::device::Device;
use crate::renderer::swapchain::Swapchain;

#[derive(Copy, Clone, Debug)]
pub struct Buffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub size: u64,
}

#[derive(Copy, Clone)]
pub struct BufferBuilder {
    size: Option<usize>,
    usage: Option<vk::BufferUsageFlags>,
    sharing_mode: Option<vk::SharingMode>,
}

impl BufferBuilder {
    pub fn new() -> BufferBuilder {
        BufferBuilder {
            size: None,
            usage: None,
            sharing_mode: None,
        }
    }

    pub fn size(&self, size: usize) -> BufferBuilder {
        BufferBuilder {
            size: Some(size),
            usage: self.usage,
            sharing_mode: self.sharing_mode,
        }
    }

    pub fn usage(&self, usage: vk::BufferUsageFlags) -> BufferBuilder {
        BufferBuilder {
            size: self.size,
            usage: Some(usage),
            sharing_mode: self.sharing_mode,
        }
    }

    pub fn sharing_mode(&self, sharing_mode: vk::SharingMode) -> BufferBuilder {
        BufferBuilder {
            size: self.size,
            usage: self.usage,
            sharing_mode: Some(sharing_mode),
        }
    }

    pub unsafe fn build(&self, c: &Core, d: &Device) -> Buffer {
        Buffer::new(c, d, self.size.expect("Error: BufferBuilder is missing size"), self.usage.expect("Error: BufferBuilder is missing usage"), self.sharing_mode.expect("Error: BufferBuilder is missing sharing_mode"))
    }

    pub unsafe fn build_many(&self, c: &Core, d: &Device, count: u32) -> Vec<Buffer> {
        let mut buffers = Vec::<Buffer>::new();
        for _ in 0..count {
            buffers.push(Buffer::new(c, d, self.size.expect("Error: BufferBuilder is missing size"), self.usage.expect("Error: BufferBuilder is missing usage"), self.sharing_mode.expect("Error: BufferBuilder is missing sharing_mode")));
        }

        buffers
    }
}

pub struct VertexAttribute {
    pub format: vk::Format,
    pub offset: usize,
}

pub trait VertexAttributes {
    fn get_attribute_data() -> Vec<VertexAttribute>;
}

pub struct VertexBuffer {
    pub binding_desc: vk::VertexInputBindingDescription,
    pub attrib_descs: Vec<vk::VertexInputAttributeDescription>,
}

impl Buffer {
    pub unsafe fn new(c: &Core, d: &Device, size: usize, usage: vk::BufferUsageFlags, sm: vk::SharingMode) -> Buffer {
        let buffer_ci = vk::BufferCreateInfo::builder()
            .size(size as u64)
            .usage(usage)
            .sharing_mode(sm);

        let buffer = d.device.create_buffer(&buffer_ci, None).unwrap();

        let memory_requirements = d.device.get_buffer_memory_requirements(buffer);
        let memory_type_index = c.instance.get_physical_device_memory_properties(d.physical_device).memory_types.iter().enumerate().find_map(|(i, m)| {
            if (memory_requirements.memory_type_bits & (1 << i)) != 0 && (m.property_flags & (vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT)
                                                                                          == (vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT)) {
                Some(i)
            } else {
                None
            }
        }).unwrap();

        let memory_alloc_i = vk::MemoryAllocateInfo::builder()
            .allocation_size(size as u64)
            .memory_type_index(memory_type_index as u32);

        let memory = d.device.allocate_memory(&memory_alloc_i, None).unwrap();
        d.device.bind_buffer_memory(buffer, memory, 0).unwrap();

        Buffer {
            buffer: buffer,
            memory: memory,
            size: size as u64,
        }
    }

    pub unsafe fn fill(&self, d: &Device, p: *const c_void, s: usize) {
        let dst = d.device.map_memory(self.memory, 0, s as u64, vk::MemoryMapFlags::empty()).unwrap();
        std::ptr::copy(p, dst, s);
        d.device.unmap_memory(self.memory);
    }
}

impl VertexBuffer {
    pub unsafe fn new<T: VertexAttributes>(verts: Vec<T>, c: &Core, d: &Device, s: &Swapchain) -> VertexBuffer {
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

        VertexBuffer {
            binding_desc: binding_desc,
            attrib_descs: attrib_descs
        }
    }
}