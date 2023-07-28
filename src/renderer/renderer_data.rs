use std::collections::HashMap;

use crate::renderer::{buffer::{Buffer, BufferBuilder}, image::{Image, ImageBuilder}, core::Core, device::Device};

pub struct RendererData {
    pub count: usize,

    pub buffers: HashMap<String, Vec<Buffer>>,
    pub images: HashMap<String, Vec<Image>>,
}

impl RendererData {
    pub unsafe fn add_buffers(&mut self, c: &Core, d: &Device, name: &str, builder: BufferBuilder) {
        self.buffers.insert(name.to_string(), builder.build_many(c, d, self.count));
    }

    pub unsafe fn add_images(&mut self, c: &Core, d: &Device, name: &str, builder: ImageBuilder) {
        self.images.insert(name.to_string(), builder.build_many(c, d, self.count));
    }

    pub fn get_buffers(&self, name: &str) -> &Vec<Buffer> {
        self.buffers.get(name).unwrap()
    }

    pub fn get_images(&self, name: &str) -> &Vec<Image> {
        self.images.get(name).unwrap()
    }
}