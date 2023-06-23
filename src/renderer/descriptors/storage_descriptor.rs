use ash::version::DeviceV1_0;
use ash::vk;

use crate::renderer::core::Core;
use crate::renderer::device::Device;
use crate::renderer::buffer::{Buffer, BufferBuilder};

#[derive(Copy, Clone)]
pub struct StorageDescriptorBuilder {
    buffer_count: usize,
    buffer_builder: BufferBuilder,
}

pub struct StorageDescriptor {
    pub buffers: Vec<Buffer>,
}

impl StorageDescriptorBuilder {
    pub fn new() -> StorageDescriptorBuilder {
        StorageDescriptorBuilder {
            buffer_count: 0,
            buffer_builder: BufferBuilder::new().usage(vk::BufferUsageFlags::STORAGE_BUFFER),
        }
    }

    pub fn buffer_count(&self, buffer_count: usize) -> StorageDescriptorBuilder {
        StorageDescriptorBuilder {
            buffer_count: buffer_count,
            buffer_builder: self.buffer_builder,
        }
    }

    pub fn buffer_sharing_mode(&self, sharing_mode: vk::SharingMode) -> StorageDescriptorBuilder {
        StorageDescriptorBuilder {
            buffer_count: self.buffer_count,
            buffer_builder: self.buffer_builder.sharing_mode(sharing_mode),
        }
    }

    pub fn buffer_size(&self, size: usize) -> StorageDescriptorBuilder {
        StorageDescriptorBuilder {
            buffer_count: self.buffer_count,
            buffer_builder: self.buffer_builder.size(size),
        }
    }

    pub unsafe fn build(&self, c: &Core, d: &Device, binding: u32, sets: &Vec<vk::DescriptorSet>) -> StorageDescriptor {
        let mut buffers = Vec::<Buffer>::new();

        for _ in 0..self.buffer_count {
            buffers.push(self.buffer_builder.build(c, d));
        }
        
        StorageDescriptor::new(d, binding, &buffers, &sets)
    }
}

impl StorageDescriptor {
    pub unsafe fn new(d: &Device, binding: u32, buffers: &Vec<Buffer>, sets: &Vec<vk::DescriptorSet>) -> StorageDescriptor {
        let mut write_sets = Vec::<vk::WriteDescriptorSet>::new();

        for i in 0..buffers.len() {
            let buffer_is = [vk::DescriptorBufferInfo::builder()
                .buffer(buffers[i].buffer)
                .range(buffers[i].size as u64)
                .build()];

            let write_set = vk::WriteDescriptorSet::builder()
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .dst_binding(binding)
                .dst_set(sets[i])
                .buffer_info(&buffer_is)
                .build();

            write_sets.push(write_set);
        }

        d.device.update_descriptor_sets(&write_sets, &[]);

        StorageDescriptor {
            buffers: buffers.clone(),
        }
    }
}