use ash::vk;

use crate::renderer::core::Core;
use crate::renderer::device::Device;
use crate::renderer::buffer::Buffer;

#[derive(Copy, Clone)]
struct BufferData {
    pub buffer: vk::Buffer,
    pub size: u64,
}

pub struct StorageDescriptorBuilder {
    buffer_datas: Option<Vec<BufferData>>,
}

#[derive(Debug)]
pub struct StorageDescriptor {}

impl StorageDescriptorBuilder {
    pub fn new() -> StorageDescriptorBuilder {
        StorageDescriptorBuilder {
            buffer_datas: None,
        }
    }

    pub fn buffers(&self, buffers: &Vec<Buffer>) -> StorageDescriptorBuilder {
        let buffer_datas = buffers.iter().map(|buffer| { BufferData { buffer: buffer.buffer, size: buffer.size} }).collect();
        StorageDescriptorBuilder {
            buffer_datas: Some(buffer_datas),
        }
    }

    pub unsafe fn build(&self, c: &Core, d: &Device, binding: u32, sets: &Vec<vk::DescriptorSet>) -> StorageDescriptor {
        StorageDescriptor::new(d, binding, self.buffer_datas.as_ref().expect("Error: Storage descriptor builder has no buffers"), sets)
    }
}

impl StorageDescriptor {
    unsafe fn new(d: &Device, binding: u32, buffers: &Vec<BufferData>, sets: &Vec<vk::DescriptorSet>) -> StorageDescriptor {
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

        StorageDescriptor {}
    }
}