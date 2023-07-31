use std::ffi::CString;

use ash::vk::{self, RenderPass};

use crate::renderer::{core::Core, image::ImageBuilder};
use crate::renderer::device::Device;
use crate::renderer::shader::Shader;
use crate::renderer::image::Image;
use crate::renderer::push_constant::PushConstant;
use crate::renderer::vertex_buffer::VertexBuffer;

pub struct GraphicsPipeline {
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    pub render_pass: RenderPass,

    pub viewport: vk::Viewport,
    pub scissor: vk::Rect2D,
    
    pub depth_image: Option<Image>,
}

impl GraphicsPipeline {
    pub unsafe fn new(c: &Core, d: &Device, target_rect: vk::Rect2D, vertex_buffer: Option<&VertexBuffer>, vertex_descriptor_set_layout: Option<vk::DescriptorSetLayout>, fragment_descriptor_set_layout: Option<vk::DescriptorSetLayout>, vertex_push_constant: Option<&PushConstant>, fragment_push_constant: Option<&PushConstant>, vs: &str, fs: &str, with_depth_buffer: bool) -> GraphicsPipeline {
        let vert_shader = Shader::new(d, vs, vk::ShaderStageFlags::VERTEX);
        let frag_shader = Shader::new(d, fs, vk::ShaderStageFlags::FRAGMENT);

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

        let viewport = vk::Viewport::builder()
            .x(target_rect.offset.x as f32)
            .y(target_rect.offset.y as f32)
            .width(target_rect.extent.width as f32)
            .height(target_rect.extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0)
            .build();

        let scissor = target_rect;

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
            .cull_mode(vk::CullModeFlags::NONE)
            //.cull_mode(vk::CullModeFlags::BACK)
            //.front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        let multisample_state_ci = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(
                vk::ColorComponentFlags::R
                | vk::ColorComponentFlags::G
                | vk::ColorComponentFlags::B
                | vk::ColorComponentFlags::A
            )
            .blend_enable(false)
            .build()
        ];

        let color_blend_state_ci = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .attachments(&color_blend_attachment_states)
            .build();

        let mut push_constant_ranges = Vec::<vk::PushConstantRange>::new();

        if let Some(push_constant) = vertex_push_constant {
            push_constant_ranges.push(vk::PushConstantRange::builder()
            .size(push_constant.size as u32)
            .offset(0)
            .stage_flags(push_constant.stage)
            .build());
        }

        if let Some(push_constant) = fragment_push_constant {
            push_constant_ranges.push(vk::PushConstantRange::builder()
            .size(push_constant.size as u32)
            .offset(0)
            .stage_flags(push_constant.stage)
            .build());
        }

        let mut descriptor_set_layouts = Vec::<vk::DescriptorSetLayout>::new();

        if let Some(descriptor_set_layout) = vertex_descriptor_set_layout {
            descriptor_set_layouts.push(descriptor_set_layout);
        }

        if let Some(descriptor_set_layout) = fragment_descriptor_set_layout {
            descriptor_set_layouts.push(descriptor_set_layout);
        }

        let (vertex_attribute_descs, vertex_binding_descs) = match vertex_buffer {
            Some(buffer) => {
                (buffer.attrib_descs.clone(), vec![buffer.binding_desc])
            },
            None => (vec![], vec![])
        };

        let vertex_input_state_ci = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&vertex_attribute_descs)
            .vertex_binding_descriptions(&vertex_binding_descs);

        let pipeline_layout_ci = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&descriptor_set_layouts)
            .push_constant_ranges(&push_constant_ranges)
            .build();
        
        let pipeline_layout = d.device.create_pipeline_layout(&pipeline_layout_ci, None).unwrap();

        let mut attachment_descs = vec![vk::AttachmentDescription {
            format: d.surface_format.format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            ..Default::default()
        }];

        let color_attachment_refs = vec![vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];

        let mut depth_stencil_state_ci_builder = vk::PipelineDepthStencilStateCreateInfo::builder();

        let mut depth_attachment_ref = None;
        let mut depth_image = None;

        if with_depth_buffer {
            let depth_format = vk::Format::D32_SFLOAT;

            depth_image = Some(ImageBuilder::new()
                .width(target_rect.extent.width)
                .height(target_rect.extent.height)
                .format(depth_format)
                .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
                .build(c, d));

            attachment_descs.push(vk::AttachmentDescription {
                format: depth_format,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::DONT_CARE,
                initial_layout: vk::ImageLayout::UNDEFINED,
                final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                ..Default::default()
            });

            depth_attachment_ref = Some(vk::AttachmentReference {
                attachment: 1,
                layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            });

            depth_stencil_state_ci_builder = depth_stencil_state_ci_builder
                .depth_test_enable(true)
                .depth_write_enable(true)
                .depth_bounds_test_enable(false)
                .stencil_test_enable(false)
                .depth_compare_op(vk::CompareOp::LESS)
        }

        let depth_stencil_state_ci = depth_stencil_state_ci_builder
            .build();

        let subpass_description_builder = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_refs);

        let subpass_description = match depth_attachment_ref.as_ref() {
            Some(depth_ref) => subpass_description_builder.depth_stencil_attachment(depth_ref).build(),
            None => subpass_description_builder.build(),
        };

        let mut subpass_dep_stage_mask = vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;
        let mut subpass_dep_access_mask = vk::AccessFlags::COLOR_ATTACHMENT_WRITE;

        if with_depth_buffer {
            subpass_dep_stage_mask |= vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS;
            subpass_dep_access_mask |= vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE;
        }
        
        let subpass_dependency = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(subpass_dep_stage_mask)
            .dst_stage_mask(subpass_dep_stage_mask)
            .dst_access_mask(subpass_dep_access_mask)
            .build();

        let render_pass_ci = vk::RenderPassCreateInfo::builder()
            .attachments(&attachment_descs)
            .subpasses(&[subpass_description])
            .dependencies(&[subpass_dependency])
            .build();

        let render_pass = d.device.create_render_pass(&render_pass_ci, None).unwrap();

        let mut pipeline_ci_builder = vk::GraphicsPipelineCreateInfo::builder()
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
            .subpass(0);

        if with_depth_buffer {
            pipeline_ci_builder = pipeline_ci_builder
                .depth_stencil_state(&depth_stencil_state_ci);
        }
    
        let pipeline_ci = pipeline_ci_builder
            .build();

        let pipeline = d.device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_ci], None).unwrap()[0];

        GraphicsPipeline {
            pipeline,
            pipeline_layout,
            render_pass,

            viewport,
            scissor,

            depth_image,
        }
    }
}