pub mod storage_descriptor;
pub mod uniform_descriptor;
pub mod image_descriptor;
pub mod sampler_descriptor;

use ash::version::DeviceV1_0;
use ash::vk;

use crate::renderer::core::Core;
use crate::renderer::device::Device;
use crate::renderer::descriptors::uniform_descriptor::UniformDescriptorBuilder;
use crate::renderer::descriptors::storage_descriptor::StorageDescriptorBuilder;
use crate::renderer::descriptors::image_descriptor::ImageDescriptorBuilder;
use crate::renderer::descriptors::sampler_descriptor::SamplerDescriptorBuilder;

pub struct DescriptorsBuilder<'a> {
    pub count: Option<usize>,
    pub stage: Option<vk::ShaderStageFlags>,
    pub uniform_builders: Vec<(u32, UniformDescriptorBuilder<'a>)>,
    pub storage_builders: Vec<(u32, StorageDescriptorBuilder)>,
    pub image_builders: Vec<(u32, ImageDescriptorBuilder<'a>)>,
    pub sampler_builders: Vec<(u32, SamplerDescriptorBuilder<'a>)>,

    next_binding: u32,
}

impl <'a> DescriptorsBuilder<'a> {
    pub fn new() -> DescriptorsBuilder<'a> {
        DescriptorsBuilder {
            count: None,
            stage: None,
            uniform_builders: Vec::<(u32, UniformDescriptorBuilder)>::new(),
            storage_builders: Vec::<(u32, StorageDescriptorBuilder)>::new(),
            image_builders: Vec::<(u32, ImageDescriptorBuilder)>::new(),
            sampler_builders: Vec::<(u32, SamplerDescriptorBuilder)>::new(),
            next_binding: 0,
        }
    }

    pub fn count(&self, count: usize) -> DescriptorsBuilder {
        DescriptorsBuilder {
            count: Some(count),
            stage: self.stage,
            uniform_builders: self.uniform_builders.clone(),
            storage_builders: self.storage_builders.clone(),
            image_builders: self.image_builders.clone(),
            sampler_builders: self.sampler_builders.clone(),
            next_binding: self.next_binding,
        }
    }

    pub fn stage(&self, stage: vk::ShaderStageFlags) -> DescriptorsBuilder {
        DescriptorsBuilder {
            count: self.count,
            stage: Some(stage),
            uniform_builders: self.uniform_builders.clone(),
            storage_builders: self.storage_builders.clone(),
            image_builders: self.image_builders.clone(),
            sampler_builders: self.sampler_builders.clone(),
            next_binding: self.next_binding,
        }
    }

    pub fn add_uniform_builder(&mut self, builder: UniformDescriptorBuilder<'a>) -> DescriptorsBuilder {
        self.uniform_builders.push((self.next_binding, builder));
        self.next_binding += 1;

        DescriptorsBuilder {
            count: self.count,
            stage: self.stage,
            uniform_builders: self.uniform_builders.clone(),
            storage_builders: self.storage_builders.clone(),
            image_builders: self.image_builders.clone(),
            sampler_builders: self.sampler_builders.clone(),
            next_binding: self.next_binding,
        }
    }

    pub fn add_storage_builder(&mut self, builder: StorageDescriptorBuilder) -> DescriptorsBuilder {
        self.storage_builders.push((self.next_binding, builder));
        self.next_binding += 1;

        DescriptorsBuilder {
            count: self.count,
            stage: self.stage,
            uniform_builders: self.uniform_builders.clone(),
            storage_builders: self.storage_builders.clone(),
            image_builders: self.image_builders.clone(),
            sampler_builders: self.sampler_builders.clone(),
            next_binding: self.next_binding,
        }
    }

    pub fn add_image_builder(&mut self, builder: ImageDescriptorBuilder<'a>) -> DescriptorsBuilder {
        self.image_builders.push((self.next_binding, builder));
        self.next_binding += 1;

        DescriptorsBuilder {
            count: self.count,
            stage: self.stage,
            uniform_builders: self.uniform_builders.clone(),
            storage_builders: self.storage_builders.clone(),
            image_builders: self.image_builders.clone(),
            sampler_builders: self.sampler_builders.clone(),
            next_binding: self.next_binding,
        }
    }

    pub fn add_sampler_builder(&mut self, builder: SamplerDescriptorBuilder<'a>) -> DescriptorsBuilder {
        self.sampler_builders.push((self.next_binding, builder));
        self.next_binding += 1;

        DescriptorsBuilder {
            count: self.count,
            stage: self.stage,
            uniform_builders: self.uniform_builders.clone(),
            storage_builders: self.storage_builders.clone(),
            image_builders: self.image_builders.clone(),
            sampler_builders: self.sampler_builders.clone(),
            next_binding: self.next_binding,
        }
    }

    pub unsafe fn build(&self, c: &Core, d: &Device) -> Descriptors {
        Descriptors::new(c, d, self)
    }
}

pub struct Descriptors {
    pub pool: vk::DescriptorPool,
    pub sets: Vec<vk::DescriptorSet>,
    pub set_layout: vk::DescriptorSetLayout,

    pub uniforms: Vec<uniform_descriptor::UniformDescriptor>,
    pub ssbos: Vec<storage_descriptor::StorageDescriptor>,
    pub images: Vec<image_descriptor::ImageDescriptor>,
    pub samplers: Vec<sampler_descriptor::SamplerDescriptor>,
}

impl Descriptors {
    pub unsafe fn new(c: &Core, d: &Device, builder: &DescriptorsBuilder) -> Descriptors {
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