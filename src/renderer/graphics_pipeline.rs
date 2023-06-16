use std::{ffi::{c_char, CString}, fs::File};

use ash::{version::DeviceV1_0, vk::VertexInputBindingDivisorDescriptionEXT, util::read_spv};
use ash::vk::{self, RenderPass};

use crate::{math::vec::{Vec2, Vec3}, renderer::shader};
use crate::renderer::core::Core;
use crate::renderer::device::Device;
use crate::renderer::swapchain::Swapchain;
use crate::renderer::shader::Shader;
use crate::renderer::image::Image2D;

pub struct GraphicsPipeline {
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    pub render_pass: RenderPass,

    pub viewport: vk::Viewport,
    pub scissor: vk::Rect2D,
}

impl GraphicsPipeline {
    pub unsafe fn new(c: &Core, d: &Device, s: &Swapchain, vs: &str, fs: &str) -> GraphicsPipeline {
        let vert_shader = Shader::new(vs, vk::ShaderStageFlags::VERTEX, c, d, s);
        let frag_shader = Shader::new(fs, vk::ShaderStageFlags::FRAGMENT, c, d, s);

        let shaders = vec![vert_shader, frag_shader];

        let shader_entry_name = CString::new("main").unwrap();

        let mut shader_stage_cis: Vec<vk::PipelineShaderStageCreateInfo> = Vec::new();

        for s in shaders.iter() {
            let shader_stage_ci = vk::PipelineShaderStageCreateInfo {
                module: s.module,
                p_name: shader_entry_name.as_ptr(),
                stage: s.flags,
                ..Default::default()
            };

            shader_stage_cis.push(shader_stage_ci);
        }

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_ci = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&dynamic_states);

        let input_assembly_state_ci = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let vertex_input_state_ci = vk::PipelineVertexInputStateCreateInfo::builder();

        let targets = &s.images;

        let viewport = vk::Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(targets[0].width as f32)
            .height(targets[0].height as f32)
            .min_depth(0.0)
            .max_depth(1.0)
            .build();

        let scissor = vk::Rect2D::builder()
            .offset(vk::Offset2D { x: 0, y: 0} )
            .extent(vk::Extent2D { width: targets[0].width, height: targets[0].height })
            .build();

        let viewport_state_ci = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .scissor_count(1)
            .viewports(&[viewport])
            .scissors(&[scissor])
            .build();

        let rasterization_state_ci = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        let multisample_state_ci = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let color_blend_attachment_state = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(
                vk::ColorComponentFlags::R
                | vk::ColorComponentFlags::G
                | vk::ColorComponentFlags::B
                | vk::ColorComponentFlags::A
            )
            .blend_enable(false)
            .build();

        let color_blend_state_ci = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .attachments(&[color_blend_attachment_state])
            .build();

        let pipeline_layout_ci = vk::PipelineLayoutCreateInfo::builder()
            .build();
        
        let pipeline_layout = d.device.create_pipeline_layout(&pipeline_layout_ci, None).unwrap();

        let render_pass_attachment = vk::AttachmentDescription {
            format: d.surface_format.format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            ..Default::default()
        };

        let color_attachment_reference = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };

        let subpass_description = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&[color_attachment_reference])
            .build();
        
        let subpass_dependency = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .build();

        let render_pass_ci = vk::RenderPassCreateInfo::builder()
            .attachments(&[render_pass_attachment])
            .subpasses(&[subpass_description])
            .dependencies(&[subpass_dependency])
            .build();

        let render_pass = d.device.create_render_pass(&render_pass_ci, None).unwrap();

        let pipeline_ci = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stage_cis)
            .input_assembly_state(&input_assembly_state_ci)
            .vertex_input_state(&vertex_input_state_ci)
            .dynamic_state(&dynamic_state_ci)
            .viewport_state(&viewport_state_ci)
            .rasterization_state(&rasterization_state_ci)
            .multisample_state(&multisample_state_ci)
            .color_blend_state(&color_blend_state_ci)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0)
            .build();

        let pipeline = d.device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_ci], None).unwrap()[0];

        GraphicsPipeline {
            pipeline: pipeline,
            pipeline_layout: pipeline_layout,
            render_pass: render_pass,

            viewport: viewport,
            scissor: scissor,
        }
    }
}