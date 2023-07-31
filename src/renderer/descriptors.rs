pub mod storage_descriptor;
pub mod uniform_descriptor;
pub mod image_descriptor;
pub mod sampler_descriptor;

use ash::vk;

use crate::renderer::{core::Core, buffer::Buffer, image::Image};
use crate::renderer::device::Device;
use crate::renderer::descriptors::uniform_descriptor::UniformDescriptorBuilder;
use crate::renderer::descriptors::storage_descriptor::StorageDescriptorBuilder;
use crate::renderer::descriptors::image_descriptor::ImageDescriptorBuilder;
use crate::renderer::descriptors::sampler_descriptor::SamplerDescriptorBuilder;

#[derive(Copy, Clone)]
pub enum DescriptorType {
    Uniform,
    Storage,
    Image,
    Sampler,
}

#[derive(Copy, Clone)]
pub enum BindingReference {
    Uniform(usize),
    Storage(usize),
    Image(usize),
    Sampler(usize),
}

#[derive(Clone)]
pub enum CreationReference {
    Uniform(String),
    Storage(String),
    Image(String),
    Sampler(String),
}

#[derive(Copy, Clone)]
pub struct DescriptorReference {
    pub descriptor_type: DescriptorType,
    pub index: usize,
}

pub struct DescriptorsBuilder {
    pub count: Option<usize>,
    pub stage: Option<vk::ShaderStageFlags>,
    pub uniform_builders: Vec<(u32, UniformDescriptorBuilder)>,
    pub storage_builders: Vec<(u32, StorageDescriptorBuilder)>,
    pub image_builders: Vec<(u32, ImageDescriptorBuilder)>,
    pub sampler_builders: Vec<(u32, SamplerDescriptorBuilder)>,

    next_binding: u32,
    pub binding_references: Vec<BindingReference>,
    pub desciptor_references: Vec<DescriptorReference>,
}

pub struct Descriptors {
    pub pool: vk::DescriptorPool,
    pub sets: Vec<vk::DescriptorSet>,
    pub set_layout: vk::DescriptorSetLayout,

    pub uniforms: Vec<uniform_descriptor::UniformDescriptor>,
    pub ssbos: Vec<storage_descriptor::StorageDescriptor>,
    pub images: Vec<image_descriptor::ImageDescriptor>,
    pub samplers: Vec<sampler_descriptor::SamplerDescriptor>,

    pub binding_references: Vec<BindingReference>,
    pub desciptor_references: Vec<DescriptorReference>,
}

impl DescriptorReference {
    pub fn new(descriptor_type: DescriptorType, index: usize) -> DescriptorReference {
        DescriptorReference { descriptor_type, index }
    }
}

impl  DescriptorsBuilder {
    pub fn new() -> DescriptorsBuilder {
        DescriptorsBuilder {
            count: None,
            stage: None,
            uniform_builders: Vec::new(),
            storage_builders: Vec::new(),
            image_builders: Vec::new(),
            sampler_builders: Vec::new(),
            next_binding: 0,
            binding_references: Vec::new(),
            desciptor_references: Vec::new(),
        }
    }

    pub fn count(mut self, count: usize) -> DescriptorsBuilder {
        self.count = Some(count);
        
        self
    }

    pub fn stage(mut self, stage: vk::ShaderStageFlags) -> DescriptorsBuilder {
        self.stage = Some(stage);
        self
    }

    pub fn add_uniform_builder(mut self, builder: UniformDescriptorBuilder) -> DescriptorsBuilder {
        self.binding_references.push(BindingReference::Uniform(self.uniform_builders.len()));
        self.desciptor_references.push(DescriptorReference::new(DescriptorType::Uniform, self.uniform_builders.len()));
        self.uniform_builders.push((self.next_binding, builder));

        self.next_binding += 1;

        self
    }

    pub fn add_storage_builder(mut self, builder: StorageDescriptorBuilder) -> DescriptorsBuilder {
        self.binding_references.push(BindingReference::Storage(self.storage_builders.len()));
        self.desciptor_references.push(DescriptorReference::new(DescriptorType::Storage, self.storage_builders.len()));
        self.storage_builders.push((self.next_binding, builder));

        self.next_binding += 1;

        self
    }

    pub fn add_image_builder(mut self, builder: ImageDescriptorBuilder) -> DescriptorsBuilder {
        self.binding_references.push(BindingReference::Image(self.image_builders.len()));
        self.desciptor_references.push(DescriptorReference::new(DescriptorType::Image, self.image_builders.len()));
        self.image_builders.push((self.next_binding, builder));

        self.next_binding += 1;

        self
    }

    pub fn add_sampler_builder(mut self, builder: SamplerDescriptorBuilder) -> DescriptorsBuilder {
        self.binding_references.push(BindingReference::Sampler(self.sampler_builders.len()));
        self.desciptor_references.push(DescriptorReference::new(DescriptorType::Sampler, self.sampler_builders.len()));
        self.sampler_builders.push((self.next_binding, builder));

        self.next_binding += 1;

        self
    }

    pub fn add_uniform_simple(self, buffers: &Vec<Buffer>) -> DescriptorsBuilder {
        self.add_uniform_builder(UniformDescriptorBuilder::new().buffers(buffers))
    }

    pub fn add_storage_simple(self, buffers: &Vec<Buffer>) -> DescriptorsBuilder {
        self.add_storage_builder(StorageDescriptorBuilder::new().buffers(buffers))
    }

    pub fn add_image_simple(self, images: &Vec<Image>) -> DescriptorsBuilder {
        self.add_image_builder(ImageDescriptorBuilder::new().images(images))
    }

    pub fn add_sampler_simple(self, images: &Vec<Image>) -> DescriptorsBuilder {
        self.add_sampler_builder(SamplerDescriptorBuilder::new().images(images))
    }

    pub unsafe fn build(self, c: &Core, d: &Device) -> Descriptors {
        Descriptors::new(c, d, self)
    }
}

impl Descriptors {
    pub unsafe fn new(c: &Core, d: &Device, builder: DescriptorsBuilder) -> Descriptors {
        let mut layout_bindings = Vec::<vk::DescriptorSetLayoutBinding>::new();

        for descriptor_builder in &builder.uniform_builders {
            layout_bindings.push(
                vk::DescriptorSetLayoutBinding::builder()
                    .binding(descriptor_builder.0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .descriptor_count(1)
                    .stage_flags(builder.stage.expect("Error: descriptors builder has no stage flags"))
                    .build()
            )
        }

        for descriptor_builder in &builder.storage_builders {
            layout_bindings.push(
                vk::DescriptorSetLayoutBinding::builder()
                    .binding(descriptor_builder.0)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .descriptor_count(1)
                    .stage_flags(builder.stage.expect("Error: descriptors builder has no stage flags"))
                    .build()
            )
        }

        for descriptor_builder in &builder.image_builders {
            layout_bindings.push(
                vk::DescriptorSetLayoutBinding::builder()
                    .binding(descriptor_builder.0)
                    .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                    .descriptor_count(1)
                    .stage_flags(builder.stage.expect("Error: descriptors builder has no stage flags"))
                    .build()
            )
        }

        for descriptor_builder in &builder.sampler_builders {
            layout_bindings.push(
                vk::DescriptorSetLayoutBinding::builder()
                    .binding(descriptor_builder.0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .descriptor_count(1)
                    .stage_flags(builder.stage.expect("Error: descriptors builder has no stage flags"))
                    .build()
            )
        }
        
        let set_layout_ci = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&layout_bindings);

        let set_layout = d.device.create_descriptor_set_layout(&set_layout_ci, None).unwrap();
        let mut set_layouts = Vec::<vk::DescriptorSetLayout>::new();

        for _ in 0..builder.count.expect("Error: descriptors builder has no count") {
            set_layouts.push(set_layout);
        }

        let temp_constant: usize = 8;
        let mut pool_sizes = Vec::<vk::DescriptorPoolSize>::new();

        if builder.uniform_builders.len() > 0 {
            pool_sizes.push(
                vk::DescriptorPoolSize::builder()
                    .ty(vk::DescriptorType::UNIFORM_BUFFER)
                    .descriptor_count((builder.uniform_builders.len() * builder.count.expect("Error: descriptors builder has no count") * temp_constant) as u32)
                    .build()
            );
        }

        if builder.storage_builders.len() > 0 {
            pool_sizes.push(
                vk::DescriptorPoolSize::builder()
                    .ty(vk::DescriptorType::STORAGE_BUFFER)
                    .descriptor_count((builder.storage_builders.len() * builder.count.expect("Error: descriptors builder has no count") * temp_constant) as u32)
                    .build()
            );
        }

        if builder.image_builders.len() > 0 {
            pool_sizes.push(
                vk::DescriptorPoolSize::builder()
                    .ty(vk::DescriptorType::STORAGE_IMAGE)
                    .descriptor_count((builder.image_builders.len() * builder.count.expect("Error: descriptors builder has no count") * temp_constant) as u32)
                    .build()
            );
        }

        if builder.sampler_builders.len() > 0 {
            pool_sizes.push(
                vk::DescriptorPoolSize::builder()
                    .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .descriptor_count((builder.sampler_builders.len() * builder.count.expect("Error: descriptors builder has no count") * temp_constant) as u32)
                    .build()
            );
        }
        
        let pool_ci = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&pool_sizes)
            .max_sets(256);

        let pool = d.device.create_descriptor_pool(&pool_ci, None).unwrap();

        let set_ai = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(pool)
            .set_layouts(&set_layouts);

        let sets = d.device.allocate_descriptor_sets(&set_ai).unwrap();

        let uniforms = Vec::<uniform_descriptor::UniformDescriptor>::new();
        let ssbos = Vec::<storage_descriptor::StorageDescriptor>::new();
        let images = Vec::<image_descriptor::ImageDescriptor>::new();
        let samplers = Vec::<sampler_descriptor::SamplerDescriptor>::new();

        let mut descriptors = Descriptors {
            pool,
            sets,
            set_layout,

            uniforms,
            ssbos,
            images,
            samplers,

            binding_references: builder.binding_references.clone(),
            desciptor_references: builder.desciptor_references.clone(),
        };

        for descriptor_builder in &builder.uniform_builders {
            descriptors.uniforms.push(descriptor_builder.1.build(c, d, descriptor_builder.0, &descriptors.sets));
        }

        for descriptor_builder in &builder.storage_builders {
            descriptors.ssbos.push(descriptor_builder.1.build(c, d, descriptor_builder.0, &descriptors.sets));
        }

        for descriptor_builder in &builder.image_builders {
            descriptors.images.push(descriptor_builder.1.build(c, d, descriptor_builder.0, &descriptors.sets));
        }

        for descriptor_builder in &builder.sampler_builders {
            descriptors.samplers.push(descriptor_builder.1.build(c, d, descriptor_builder.0, &descriptors.sets));
        }

        descriptors
    }

    pub unsafe fn bind(&self, d: &Device, b: &vk::CommandBuffer, bp: vk::PipelineBindPoint, pl: &vk::PipelineLayout, i: usize) {
        d.device.cmd_bind_descriptor_sets(*b, bp, *pl, 0, &[self.sets[i]], &[]);
    }
}