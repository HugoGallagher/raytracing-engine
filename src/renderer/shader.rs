use std::fs::File;

use ash::util::read_spv;
use ash::vk;

use crate::renderer::core::Core;
use crate::renderer::device::Device;
use crate::renderer::swapchain::Swapchain;

pub struct Shader {
    pub module: vk::ShaderModule,
    pub flags: vk::ShaderStageFlags,
    pub bytecode: Vec<u32>,
}

impl Shader {
    pub unsafe fn new(d: &Device, path: &str, flags: vk::ShaderStageFlags) -> Shader {
        let shader_base_path = "./res/shaders/bin/";
        let shader_ext = ".spv";

        let mut shader_path = String::from(shader_base_path);
        shader_path.push_str(path);
        shader_path.push_str(shader_ext);

        let shader_path = shader_path.as_str();

        let mut shader_file = File::open(shader_path).expect(format!("Error: Shader file at {shader_path} doesn't exist").as_str());
        let bytecode = read_spv(&mut shader_file).expect("Error reading shader");
        let shader_ci = vk::ShaderModuleCreateInfo::builder().code(&bytecode);
        let module = d.device.create_shader_module(&shader_ci, None).expect("Error creating shader module");

        Shader {
            module,
            flags,
            bytecode,
        }
    }
}