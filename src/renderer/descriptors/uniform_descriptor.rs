use ash::version::DeviceV1_0;
use ash::vk;

use crate::renderer::core::Core;
use crate::renderer::device::Device;
use crate::renderer::buffer::{Buffer, BufferBuilder};

#[derive(Copy, Clone)]
pub struct UniformDescriptorBuilder {
    buffer_count: usize,
    buffer_builder: BufferBuilder,
}

pub struct UniformDescriptor {
    pub buffers: Vec<Buffer>,
}

impl UniformDescriptorBuilder {
    pub fn new() -> UniformDescriptorBuilder {
        UniformDescriptorBuilder {
            buffer_count: 0,
            buffer_builder: BufferBuilder::new(),
        }
    }

    pub fn buffer_count(&self, buffer_count: usize) -> UniformDescriptorBuilder {
        UniformDescriptorBuilder {
            buffer_count: buffer_count,
            buffer_builder: self.buffer_builder,
        }
    }

    pub fn buffer_builder(&self, buffer_builder: BufferBuilder) -> UniformDescriptorBuilder {
        UniformDescriptorBuilder {
            buffer_count: self.buffer_count,
            buffer_builder: buffer_builder,
        }
    }

    pub unsafe fn build(&self, c: &Core, d: &Device, binding: u32, sets: &Vec<vk::DescriptorSet>) -> UniformDescriptor {
        let mut buffers = Vec::<Buffer>::new();

        for _ in 0..self.buffer_count {
            buffers.push(self.buffer_builder.build(c, d));
        }
        
        UniformDescriptor::new(d, binding, &buffers, &sets)
    }
}

impl UniformDescriptor {
    pub unsafe fn new(d: &Device, binding: u32, buffers: &Vec<Buffer>, sets: &Vec<vk::DescriptorSet>) -> UniformDescriptor {
        let mut write_sets = Vec::<vk::WriteDescriptorSet>::new();

        for i in 0..buffers.len() {
            let buffer_is = [vk::DescriptorBufferInfo::builder()
                .buffer(buffers[i].buffer)
                .range(buffers[i].size as u64)
                .build()];

            let write_set = vk::WriteDescriptorSet::builder()
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .dst_binding(binding)
                .dst_set(sets[i])
                .buffer_info(&buffer_is)
                .build();

            write_sets.push(write_set);
        }

        d.device.update_descriptor_sets(&write_sets, &[]);

        UniformDescriptor {
            buffers: buffers.clone(),
        }
    }
}