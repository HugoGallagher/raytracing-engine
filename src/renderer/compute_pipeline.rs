use std::ffi::CString;

use ash::vk;

use crate::renderer::core::Core;
use crate::renderer::device::Device;
use crate::renderer::push_constant::PushConstant;
use crate::renderer::shader::Shader;

pub struct ComputePipeline {
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
}

impl ComputePipeline {
    pub unsafe fn new(c: &Core, d: &Device, descriptor_set_layout: Option<vk::DescriptorSetLayout>, push_constant: Option<&PushConstant>, cs: &str) -> ComputePipeline {
        let comp_shader = Shader::new(d, cs, vk::ShaderStageFlags::COMPUTE);

        let shader_entry_name = CString::new("main").unwrap();

        let shader_stage_ci = vk::PipelineShaderStageCreateInfo::builder()
            .module(comp_shader.module)
            .name(&shader_entry_name)
            .stage(vk::ShaderStageFlags::COMPUTE)
            .build();

        let push_constant_ranges = match push_constant {
            Some(pc) => {
                vec![vk::PushConstantRange::builder()
                    .size(pc.size as u32)
                    .offset(0)
                    .stage_flags(pc.stage)
                    .build()]
            },
            None => vec![]
        };

        let descriptor_set_layouts = match descriptor_set_layout {
            Some(layout) => {
                vec![layout]
            },
            None => vec![]
        };

        let pipeline_layout_ci = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&descriptor_set_layouts)
            .push_constant_ranges(&push_constant_ranges)
            .build();

        let pipeline_layout = d.device.create_pipeline_layout(&pipeline_layout_ci, None).unwrap();

        let pipeline_ci = vk::ComputePipelineCreateInfo::builder()
            .stage(shader_stage_ci)
            .layout(pipeline_layout)
            .build();

        let pipeline = d.device.create_compute_pipelines(vk::PipelineCache::null(), &[pipeline_ci], None).unwrap()[0];

        ComputePipeline {
            pipeline: pipeline,
            pipeline_layout: pipeline_layout,
        }
    }
}