use ash::version::DeviceV1_0;
use ash::vk;

use crate::renderer::core::Core;
use crate::renderer::device::Device;
use crate::renderer::image::Image2D;
use crate::renderer::commands::Commands;
use crate::renderer::descriptors::Descriptors;
use crate::renderer::swapchain::Swapchain;

pub struct ImageDescriptorBuilder {

}

pub struct ImageDescriptor {
    pub images: Vec<Image2D>,
}

impl ImageDescriptor {
    pub unsafe fn new(p: &Descriptors, b: u32, ims: &Vec<Image2D>, c: &Core, d: &Device, s: &Swapchain) -> ImageDescriptor {
        let mut write_sets = Vec::<vk::WriteDescriptorSet>::new();

        for i in 0..s.image_count {
            let image_is = [vk::DescriptorImageInfo::builder()
                .image_view(ims[i as usize].view)
                .image_layout(vk::ImageLayout::GENERAL)
                .build()];

            let write_set = vk::WriteDescriptorSet::builder()
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .dst_binding(b)
                .dst_set(p.sets[i as usize])
                .image_info(&image_is)
                .build();

            write_sets.push(write_set);
        }

        d.device.update_descriptor_sets(&write_sets, &[]);

        let layout_transition_buffer = Commands::new(d, s, d.queue_compute.1, s.image_count);

        layout_transition_buffer.record_all(d, |i, b| {
            let subresource_range = vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .layer_count(1)
                .level_count(1)
                .build();

            let barrier = vk::ImageMemoryBarrier::builder()
                .image(ims[i].image)
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
            images: ims.clone(),
        }
    }
}