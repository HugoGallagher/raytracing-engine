use ash::version::DeviceV1_0;
use ash::vk;

use crate::renderer::{core::Core, image};
use crate::renderer::device::Device;
use crate::renderer::image::{Image2D, Image2DBuilder};
use crate::renderer::commands::Commands;
use crate::renderer::descriptors::Descriptors;

use super::sampler_descriptor::{SamplerDescriptor, self};

#[derive(Copy, Clone)]
pub struct ImageDescriptorBuilder<'a> {
    image_count: usize,
    image_builder: Image2DBuilder,
    images: Option<&'a Vec<Image2D>>,
}

pub struct ImageDescriptor {
    pub images: Vec<Image2D>,
}

impl <'a> ImageDescriptorBuilder<'a> {
    pub fn new() -> ImageDescriptorBuilder<'a> {
        ImageDescriptorBuilder {
            image_count: 0,
            image_builder: Image2DBuilder::new(),
            images: None,
        }
    }

    pub fn image_count(&self, image_count: usize) -> ImageDescriptorBuilder<'a> {
        ImageDescriptorBuilder {
            image_count: image_count,
            image_builder: self.image_builder,
            images: self.images,
        }
    }

    pub fn image_builder(&self, image_builder: Image2DBuilder) -> ImageDescriptorBuilder<'a> {
        ImageDescriptorBuilder {
            image_count: self.image_count,
            image_builder: image_builder,
            images: self.images,
        }
    }

    pub fn images(&self, images: &'a Vec<Image2D>) -> ImageDescriptorBuilder<'a> {
        ImageDescriptorBuilder {
            image_count: self.image_count,
            image_builder: self.image_builder,
            images: Some(images),
        }
    }

    pub unsafe fn build(&self, c: &Core, d: &Device, binding: u32, sets: &Vec<vk::DescriptorSet>) -> ImageDescriptor {
        if self.images.is_none() {
            let mut images = Vec::<Image2D>::new();

            for _ in 0..self.image_count {
                images.push(self.image_builder.build(c, d));
            };

            ImageDescriptor::new(c, d, binding, &images, sets)
        } else {
            ImageDescriptor::new(c, d, binding, self.images.unwrap(), sets)
        }
    }
}

impl ImageDescriptor {
    pub unsafe fn new(c: &Core, d: &Device, binding: u32, images: &Vec<Image2D>, sets: &Vec<vk::DescriptorSet>) -> ImageDescriptor {
        let mut write_sets = Vec::<vk::WriteDescriptorSet>::new();

        for i in 0..images.len() {
            let image_is = [vk::DescriptorImageInfo::builder()
                .image_view(images[i as usize].view)
                .image_layout(vk::ImageLayout::GENERAL)
                .build()];

            let write_set = vk::WriteDescriptorSet::builder()
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .dst_binding(binding)
                .dst_set(sets[i as usize])
                .image_info(&image_is)
                .build();

            write_sets.push(write_set);
        }

        d.device.update_descriptor_sets(&write_sets, &[]);

        let layout_transition_buffer = Commands::new(d, d.queue_compute.1, images.len());

        layout_transition_buffer.record_all(d, |i, b| {
            let subresource_range = vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .layer_count(1)
                .level_count(1)
                .build();

            let barrier = vk::ImageMemoryBarrier::builder()
                .image(images[i].image)
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::GENERAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .subresource_range(subresource_range)
                .build();

            d.device.cmd_pipeline_barrier(b, vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::TRANSFER, vk::DependencyFlags::empty(), &[], &[], &[barrier]);
        });

        let submit_is = [vk::SubmitInfo::builder()
            .command_buffers(&layout_transition_buffer.buffers)
            .build()];

        d.device.queue_submit(d.queue_compute.0, &submit_is, vk::Fence::null()).unwrap();
        d.device.queue_wait_idle(d.queue_compute.0).unwrap();

        ImageDescriptor {
            images: images.clone(),
        }
    }
}