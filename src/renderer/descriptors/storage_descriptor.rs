use ash::version::DeviceV1_0;
use ash::vk;

use crate::renderer::core::Core;
use crate::renderer::device::Device;
use crate::renderer::buffer::Buffer;

pub struct StorageDescriptorBuilder {
    binding: u32,
    buffers: Vec<Buffer>,
    sets: Vec<vk::DescriptorSet>,
}

pub struct StorageDescriptor {
    pub buffers: Vec<Buffer>,
}

impl StorageDescriptorBuilder {
    pub fn new() -> StorageDescriptorBuilder {
        StorageDescriptorBuilder {
            binding: 0,
            buffers: Vec::<Buffer>::new(),
            sets: Vec::<vk::DescriptorSet>::new(),
        }
    }

    pub unsafe fn build(&self, c: &Core, d: &Device) -> StorageDescriptor {
        StorageDescriptor::new(d, self.binding, &self.buffers, &self.sets)
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