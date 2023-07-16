use std::ffi::c_void;

use ash::vk;

use crate::renderer::core::Core;
use crate::renderer::device::Device;

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
impl Buffer {
    pub unsafe fn new(c: &Core, d: &Device, size: usize, usage: vk::BufferUsageFlags, sm: vk::SharingMode) -> Buffer {
        let buffer_ci = vk::BufferCreateInfo::builder()
            .size(size as u64)
            .usage(usage)
            .sharing_mode(sm);

        let buffer = d.device.create_buffer(&buffer_ci, None).unwrap();

        let memory_type_index = d.get_memory_type(c, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT, buffer);

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