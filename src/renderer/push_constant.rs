use std::{ptr, mem};

use ash::vk;

pub struct PushConstantBuilder {
    size: usize,
    stage: Option<vk::ShaderStageFlags>,
}

pub struct PushConstant {
    pub data: Vec<u8>,
    pub size: usize,
    pub stage: vk::ShaderStageFlags,
}

impl PushConstantBuilder {
    pub fn new() -> PushConstantBuilder {
        PushConstantBuilder {
            size: 0,
            stage: None,
        }
    }

    pub fn size(mut self, size: usize) -> PushConstantBuilder {
        self.size = size;
        self
    }

    pub fn stage(mut self, stage: vk::ShaderStageFlags) -> PushConstantBuilder {
        self.stage = Some(stage);
        self
    }

    pub fn build(&self) -> PushConstant {
        PushConstant::new(self.size, self.stage.unwrap())
    }
}

impl PushConstant {
    pub fn new(size: usize, stage: vk::ShaderStageFlags) -> PushConstant {
        let mut data = Vec::<u8>::new();
        data.resize(size, 0);

        PushConstant {
            data,
            size,
            stage,
        }
    }

    pub unsafe fn set_data<T>(&mut self, data: &T) {
        assert!(mem::size_of::<T>() <= 128, "Error: Push constant data type is larger than 128 bytes");

        self.data.resize(mem::size_of::<T>(), 0);
        
        let data_ptr = self.data.as_mut_ptr();
        
        ptr::copy(data as *const T as *mut u8, data_ptr, self.size);
    }
}