use std::fs::File;
use std::{ffi::CString, io::Cursor};

use ash::{util::read_spv, version::DeviceV1_0, vk};

use crate::renderer::device::Device;
use crate::renderer::descriptors::Descriptors;
use crate::renderer::shader::Shader;

pub struct ComputePipeline {
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
}

impl ComputePipeline {
    pub unsafe fn new(d: &Device, de: &Descriptors, cs: &str) -> ComputePipeline {
        let comp_shader = Shader::new(d, cs, vk::ShaderStageFlags::COMPUTE);

        let shader_entry_name = CString::new("main").unwrap();

        let shader_stage_ci = vk::PipelineShaderStageCreateInfo::builder()
            .module(comp_shader.module)
            .name(&shader_entry_name)
            .stage(vk::ShaderStageFlags::COMPUTE)
            .build();

        let pipeline_layout_ci = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&[de.set_layout])
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