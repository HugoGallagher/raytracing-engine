use ash::vk;

use crate::renderer::{core::Core, image};
use crate::renderer::device::Device;
use crate::renderer::image::{Image2D, Image2DBuilder};
use crate::renderer::commands::Commands;
use crate::renderer::descriptors::Descriptors;

struct ImageData {
    image: vk::Image,
    view: vk::ImageView,
}

pub struct ImageDescriptorBuilder {
    image_datas: Option<Vec<ImageData>>,
}

pub struct ImageDescriptor {}

impl  ImageDescriptorBuilder {
    pub fn new() -> ImageDescriptorBuilder {
        ImageDescriptorBuilder {
            image_datas: None,
        }
    }

    pub fn images(&self, images: &Vec<Image2D>) -> ImageDescriptorBuilder {
        let image_datas = images.iter().map(|image| { ImageData { image: image.image, view: image.view} }).collect();
        ImageDescriptorBuilder {
            image_datas: Some(image_datas),
        }
    }

    pub unsafe fn build(&self, c: &Core, d: &Device, binding: u32, sets: &Vec<vk::DescriptorSet>) -> ImageDescriptor {
        ImageDescriptor::new(c, d, binding, &self.image_datas.as_ref().expect("Error: Image descriptor builder has no images"), sets)
    }
}

impl ImageDescriptor {
    unsafe fn new(c: &Core, d: &Device, binding: u32, images: &Vec<ImageData>, sets: &Vec<vk::DescriptorSet>) -> ImageDescriptor {
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

        let layout_transition_buffer = Commands::new(d, d.queue_compute.1, images.len(), false);

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

        ImageDescriptor {}
    }
}