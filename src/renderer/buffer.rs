use std::ffi::c_void;

use ash::vk;

use crate::renderer::{core::Core, commands::Commands};
use crate::renderer::device::Device;

#[derive(Copy, Clone)]
pub struct BufferBuilder {
    size: Option<usize>,
    usage: Option<vk::BufferUsageFlags>,
    sharing_mode: Option<vk::SharingMode>,
    properties: Option<vk::MemoryPropertyFlags>,
}

#[derive(Copy, Clone, Debug)]
pub struct Buffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub size: u64,
    pub host_visible: bool,
}

impl BufferBuilder {
    pub fn new() -> BufferBuilder {
        BufferBuilder {
            size: None,
            usage: None,
            sharing_mode: None,
            properties: None,
        }
    }

    pub fn size(&self, size: usize) -> BufferBuilder {
        BufferBuilder {
            size: Some(size),
            usage: self.usage,
            sharing_mode: self.sharing_mode,
            properties: self.properties,
        }
    }

    pub fn usage(&self, usage: vk::BufferUsageFlags) -> BufferBuilder {
        BufferBuilder {
            size: self.size,
            usage: Some(usage),
            sharing_mode: self.sharing_mode,
            properties: self.properties,
        }
    }

    pub fn sharing_mode(&self, sharing_mode: vk::SharingMode) -> BufferBuilder {
        BufferBuilder {
            size: self.size,
            usage: self.usage,
            sharing_mode: Some(sharing_mode),
            properties: self.properties,
        }
    }

    pub fn properties(&self, properties: vk::MemoryPropertyFlags) -> BufferBuilder {
        BufferBuilder {
            size: self.size,
            usage: self.usage,
            sharing_mode: self.sharing_mode,
            properties: Some(properties),
        }
    }

    pub unsafe fn build(&self, c: &Core, d: &Device) -> Buffer {
        Buffer::new(c, d, None, self.size.expect("Error: BufferBuilder is missing size"), self.usage.expect("Error: BufferBuilder is missing usage"), self.sharing_mode.expect("Error: BufferBuilder is missing sharing_mode"), self.properties.expect("Error: BufferBuilder is missing property"))
    }

    pub unsafe fn build_many(&self, c: &Core, d: &Device, count: usize) -> Vec<Buffer> {
        let mut buffers = Vec::<Buffer>::new();
        for _ in 0..count {
            buffers.push(Buffer::new(c, d, None, self.size.expect("Error: BufferBuilder is missing size"), self.usage.expect("Error: BufferBuilder is missing usage"), self.sharing_mode.expect("Error: BufferBuilder is missing sharing_mode"), self.properties.expect("Error: BufferBuilder is missing property")));
        }

        buffers
    }

    pub unsafe fn build_with_data(&self, c: &Core, d: &Device, data: *const c_void) -> Buffer {
        Buffer::new(c, d, Some(data), self.size.expect("Error: BufferBuilder is missing size"), self.usage.expect("Error: BufferBuilder is missing usage"), self.sharing_mode.expect("Error: BufferBuilder is missing sharing_mode"), self.properties.expect("Error: BufferBuilder is missing property"))
    }

    pub unsafe fn build_many_with_data(&self, c: &Core, d: &Device, data: Vec<*const c_void>, count: usize) -> Vec<Buffer> {
        let mut buffers = Vec::<Buffer>::new();
        for i in 0..count {
            buffers.push(Buffer::new(c, d, Some(data[i]), self.size.expect("Error: BufferBuilder is missing size"), self.usage.expect("Error: BufferBuilder is missing usage"), self.sharing_mode.expect("Error: BufferBuilder is missing sharing_mode"), self.properties.expect("Error: BufferBuilder is missing property")));
        }

        buffers
    }
}
impl Buffer {
    pub unsafe fn new(c: &Core, d: &Device, data: Option<*const c_void>, size: usize, usage: vk::BufferUsageFlags, sm: vk::SharingMode, properties: vk::MemoryPropertyFlags) -> Buffer {
        let host_visible = properties & vk::MemoryPropertyFlags::HOST_VISIBLE == vk::MemoryPropertyFlags::HOST_VISIBLE;
        
        let mut usage = usage;

        let mut staging_buffer: Option<Buffer> = None;

        if !host_visible {
            assert!(data.is_some(), "Buffer is not device local but no data is provided");

            staging_buffer = Some(BufferBuilder::new()
                .size(size)
                .usage(vk::BufferUsageFlags::TRANSFER_SRC)
                .sharing_mode(sm)
                .properties(vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT)
                .build(c, d));

            usage |= vk::BufferUsageFlags::TRANSFER_DST;

            staging_buffer.unwrap().fill_from_ptr(d, data.unwrap(), size);
        }
        
        let buffer_ci = vk::BufferCreateInfo::builder()
            .size(size as u64)
            .usage(usage)
            .sharing_mode(sm);

        let buffer = d.device.create_buffer(&buffer_ci, None).unwrap();

        let memory_requirements = d.device.get_buffer_memory_requirements(buffer);
        let memory_type_index = d.get_memory_type(c, properties, memory_requirements);

        let memory_alloc_i = vk::MemoryAllocateInfo::builder()
            .allocation_size(size as u64)
            .memory_type_index(memory_type_index as u32);

        let memory = d.device.allocate_memory(&memory_alloc_i, None).unwrap();
        d.device.bind_buffer_memory(buffer, memory, 0).unwrap();

        let buffer = Buffer {
            buffer,
            memory,
            size: size as u64,
            host_visible,
        };

        if data.is_some() && host_visible {
            buffer.fill_from_ptr(d, data.unwrap(), size);
        }

        if !host_visible {
            let transfer_commands = Commands::new(d, d.queue_main.1, 1, true);

            transfer_commands.record_one(d, 0, |b| {
                let buffer_copy = vk::BufferCopy::builder()
                    .size(size as u64)
                    .build();

                d.device.cmd_copy_buffer(b, staging_buffer.unwrap().buffer, buffer.buffer, &[buffer_copy])
            });

            let submit_i = vk::SubmitInfo::builder()
                .command_buffers(&[transfer_commands.buffers[0]])
                .build();

            d.device.queue_submit(d.queue_main.0, &[submit_i], vk::Fence::null()).unwrap();
            d.device.queue_wait_idle(d.queue_main.0).unwrap();
        }

        buffer
    }

    pub unsafe fn fill<T>(&self, d: &Device, data: &Vec<T>) {
        let size = data.len() * std::mem::size_of::<T>();
        let dst = d.device.map_memory(self.memory, 0, size as u64, vk::MemoryMapFlags::empty()).unwrap();
        std::ptr::copy(data.as_ptr() as *const c_void, dst, size);
        d.device.unmap_memory(self.memory);
    }

    pub unsafe fn fill_from_ptr(&self, d: &Device, p: *const c_void, s: usize) {
        let dst = d.device.map_memory(self.memory, 0, s as u64, vk::MemoryMapFlags::empty()).unwrap();
        std::ptr::copy(p, dst, s);
        d.device.unmap_memory(self.memory);
    }
}