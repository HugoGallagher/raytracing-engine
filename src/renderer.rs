pub mod core;
pub mod device;
pub mod swapchain;
pub mod buffer;
pub mod image;
pub mod sampler;
pub mod vertex_buffer;
pub mod descriptors;
pub mod shader;
pub mod framebuffer;
pub mod commands;
pub mod compute_pipeline;
pub mod graphics_pipeline;
pub mod compute_pass;
pub mod graphics_pass;
pub mod compute_layer;
pub mod graphics_layer;
pub mod fence;
pub mod semaphore;
pub mod frame;
pub mod mesh;
pub mod push_constant;

use std::{mem, ffi::c_void, collections::HashMap};

use ash::vk;
use raw_window_handle::{RawWindowHandle, RawDisplayHandle};

use crate::{math::{vec::{Vec4, Vec3, Vec2}, mat::Mat4}, renderer::{descriptors::{storage_descriptor, image_descriptor, sampler_descriptor, uniform_descriptor}, mesh::Tri, compute_pass::ComputePassDispatchInfo, graphics_pass::GraphicsPassDrawInfo}};

#[repr(C)]
pub struct PushConstantData {
    pub view: Mat4,
    pub pos: Vec3,
    pub downscale: u32,
    pub tri_count: u32,
}

#[repr(C)]
pub struct MeshPushConstant {
    pub view_proj: Mat4,
    pub model: Mat4,
}

#[repr(C)]
pub struct Vertex {
    pub pos: Vec2,
}

impl vertex_buffer::VertexAttributes for Vertex {
    fn get_attribute_data() -> Vec<vertex_buffer::VertexAttribute> {
        vec![vertex_buffer::VertexAttribute { format: vk::Format::R32G32_SFLOAT, offset: 0 }]
    }
}

#[repr(C)]
pub struct MeshVertex {
    pub pos: Vec3,
    pub col: Vec3,
}

impl vertex_buffer::VertexAttributes for MeshVertex {
    fn get_attribute_data() -> Vec<vertex_buffer::VertexAttribute> {
        vec![
            vertex_buffer::VertexAttribute { format: vk::Format::R32G32B32_SFLOAT, offset: 0 },
            vertex_buffer::VertexAttribute { format: vk::Format::R32G32B32_SFLOAT, offset: 12 },
        ]
    }
}


pub struct Renderer {
    core: core::Core,
    device: device::Device,
    swapchain: swapchain::Swapchain,

    buffers: HashMap<String, Vec<buffer::Buffer>>,
    images: HashMap<String, Vec<image::Image>>,

    pub push_constant: PushConstantData,
    pub mesh_push_constant: MeshPushConstant,

    pub compute_layer: compute_layer::ComputeLayer,
    pub graphics_layer: graphics_layer::GraphicsLayer,

    frames: Vec<frame::Frame>,

    tris: Vec<Tri>,

    frames_in_flight: usize,
    current_frame: usize,
}

impl Renderer {
    pub unsafe fn new(window: RawWindowHandle, display: RawDisplayHandle) -> Renderer {
        const FRAMES_IN_FLIGHT: u32 = 2;

        const MAX_TRIS: usize = 8192;
        const DOWNSCALE: u32 = 2;

        let debug = true;

        let mut tris = Vec::<Tri>::with_capacity(MAX_TRIS);
        
        mesh::parse_obj(&mut tris, "res/meshes/asdf.obj");

        let push_constant = PushConstantData {
            view: Mat4::identity(),
            pos: Vec3::zero(),
            downscale: DOWNSCALE,
            tri_count: tris.len() as u32,
        };

        let mesh_push_constant = MeshPushConstant {
            view_proj: Mat4::identity(),
            model: Mat4::identity(),
        };

        let core = core::Core::new(debug, display);
        let device = device::Device::new(&core, window, display);
        let swapchain = swapchain::Swapchain::new(&core, &device);

        let mut buffers = HashMap::<String, Vec<buffer::Buffer>>::new();
        let mut images = HashMap::<String, Vec<image::Image>>::new();

        let mut compute_layer = compute_layer::ComputeLayer::new(&core, &device, FRAMES_IN_FLIGHT as usize);
        let mut graphics_layer = graphics_layer::GraphicsLayer::new(&core, &device, FRAMES_IN_FLIGHT as usize);

        let buffer_builder = buffer::BufferBuilder::new()
            .size(mem::size_of::<Tri>() * MAX_TRIS)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .usage(vk::BufferUsageFlags::STORAGE_BUFFER)
            .properties(vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);

        let image_builder = image::ImageBuilder::new()
            .width(1280 / DOWNSCALE)
            .height(720 / DOWNSCALE)
            .usage(vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED)
            .format(vk::Format::R8G8B8A8_UNORM);

        buffers.insert("tris".to_string(), buffer_builder.build_many(&core, &device, FRAMES_IN_FLIGHT as usize));
        images.insert("raytraced_image".to_string(), image_builder.build_many(&core, &device, FRAMES_IN_FLIGHT as usize));

        let image_descriptor_builder = descriptors::image_descriptor::ImageDescriptorBuilder::new()
            .images(&images.get("raytraced_image").unwrap());
            
        let sampler_descriptor_builder = descriptors::sampler_descriptor::SamplerDescriptorBuilder::new()
            .images(&images.get("raytraced_image").unwrap());

        let storage_descriptor_builder = descriptors::storage_descriptor::StorageDescriptorBuilder::new()
            .buffers(&buffers.get("tris").unwrap());

        let compute_descriptors_builder = descriptors::DescriptorsBuilder::new()
            .count(FRAMES_IN_FLIGHT as usize)
            .stage(vk::ShaderStageFlags::COMPUTE)
            .add_storage_builder(storage_descriptor_builder)
            .add_image_builder(image_descriptor_builder);

        let compute_push_constant_builder = push_constant::PushConstantBuilder::new()
            .size(mem::size_of::<PushConstantData>())
            .stage(vk::ShaderStageFlags::COMPUTE);

        let compute_pass_dispatch_info = ComputePassDispatchInfo {
            x: (1280 / 16) / DOWNSCALE + 1,
            y: (720 / 16) / DOWNSCALE + 1,
            z: 1
        };

        compute_layer.add_pass(&core, &device, Some(compute_descriptors_builder), Some(compute_push_constant_builder), "raytracer.comp", compute_pass_dispatch_info);

        let graphics_descriptors_builder = descriptors::DescriptorsBuilder::new()
            .count(FRAMES_IN_FLIGHT as usize)
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .add_sampler_builder(sampler_descriptor_builder);

        let mut mesh_tris = Vec::<Tri>::with_capacity(2048);
        mesh::parse_obj(&mut mesh_tris, "res/meshes/torus.obj");

        let mut mesh_verts = Vec::<MeshVertex>::with_capacity(2048);

        for tri in mesh_tris {
            mesh_verts.push(MeshVertex { pos: tri.verts[0].to_vec3(), col: tri.normal.to_vec3() });
            mesh_verts.push(MeshVertex { pos: tri.verts[1].to_vec3(), col: tri.normal.to_vec3() });
            mesh_verts.push(MeshVertex { pos: tri.verts[2].to_vec3(), col: tri.normal.to_vec3() });
        }

        let mesh_pass_draw_info = GraphicsPassDrawInfo {
            vertex_count: mesh_verts.len() as u32,
            index_count: 0,
            instance_count: 1,
            first_vertex: 0,
            first_instance: 0,
            vertex_offset: 0,
        };

        let quad_verts = vec![
            Vertex { pos: Vec2::new(-1.0, -1.0) },
            Vertex { pos: Vec2::new(1.0, -1.0) },
            Vertex { pos: Vec2::new(1.0, 1.0) },
            Vertex { pos: Vec2::new(-1.0, 1.0) },
        ];

        let quad_indices = vec![0, 1, 2, 2, 3, 0];

        let quad_pass_draw_info = GraphicsPassDrawInfo {
            vertex_count: quad_verts.len() as u32,
            index_count: quad_indices.len() as u32,
            instance_count: 1,
            first_vertex: 0,
            first_instance: 0,
            vertex_offset: 0,
        };

        let mesh_push_constant_builder = push_constant::PushConstantBuilder::new()
            .size(mem::size_of::<MeshPushConstant>())
            .stage(vk::ShaderStageFlags::VERTEX);

        graphics_layer.add_pass(&core, &device, &swapchain.images, None, None, Some(&quad_verts), Some(&quad_indices), Some(graphics_descriptors_builder), None, "draw_to_screen.vert", "draw_to_screen.frag", false ,quad_pass_draw_info);
        graphics_layer.add_pass(&core, &device, &swapchain.images, Some(vk::Extent2D { width: 320, height: 180 }), None, Some(&mesh_verts), None, None, Some(mesh_push_constant_builder), "mesh.vert", "mesh.frag", true, mesh_pass_draw_info);
        
        let mut frames = Vec::<frame::Frame>::new();

        for _ in 0..FRAMES_IN_FLIGHT {
            frames.push(frame::Frame::new(&device));
        }

        Renderer {
            core,
            device,
            swapchain,

            buffers,
            images,

            push_constant,
            mesh_push_constant,

            compute_layer,
            graphics_layer,

            frames,
            tris,

            frames_in_flight: FRAMES_IN_FLIGHT as usize,
            current_frame: 0,
        }
    }

    pub unsafe fn add_buffer(&mut self, name: &str, builder: buffer::BufferBuilder) {
        self.buffers.insert(name.to_string(), builder.build_many(&self.core, &self.device, self.frames_in_flight));
    }

    pub unsafe fn add_image(&mut self, name: &str, builder: image::ImageBuilder) {
        self.images.insert(name.to_string(), builder.build_many(&self.core, &self.device, self.frames_in_flight));
    }

    pub unsafe fn draw(&mut self) {
        self.current_frame = (self.current_frame + 1) % self.frames_in_flight;

        let active_frame = &self.frames[self.current_frame];

        self.device.device.wait_for_fences(&[active_frame.in_flight_fence.fence], true, u64::MAX).unwrap();
        self.device.device.reset_fences(&[active_frame.in_flight_fence.fence]).unwrap();

        let present_index = self.swapchain.swapchain_init.acquire_next_image(self.swapchain.swapchain, u64::MAX, active_frame.image_available_semaphore.semaphore, vk::Fence::null()).unwrap().0 as usize;
        let present_indices = [present_index as u32];
        
        self.compute_layer.fill_push_constant(0, &self.push_constant);
        self.graphics_layer.fill_push_constant(1, &self.mesh_push_constant);

        self.buffers.get("tris").unwrap()[self.current_frame].fill(&self.device, self.tris.as_ptr() as *const c_void, self.tris.len() * mem::size_of::<Tri>());

        self.compute_layer.record_one(&self.device, self.current_frame);
        self.graphics_layer.record_one(&self.device, self.current_frame, present_index);

        let compute_wait_semaphores = [];
        let compute_signal_semaphores = [active_frame.compute_finished_semaphore.semaphore];
        let compute_wait_stages = [];

        let graphics_wait_semaphores = [active_frame.image_available_semaphore.semaphore, active_frame.compute_finished_semaphore.semaphore];
        let graphics_signal_semaphores = [active_frame.render_finished_semaphore.semaphore];
        let graphics_wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT, vk::PipelineStageFlags::FRAGMENT_SHADER];

        let compute_command_buffers = [self.compute_layer.commands.buffers[self.current_frame]];
        let graphics_command_buffers = [self.graphics_layer.commands.buffers[self.current_frame]];

        let compute_submit_i = vk::SubmitInfo {
            wait_semaphore_count: 0,
            p_wait_semaphores: compute_wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: compute_wait_stages.as_ptr(),
            signal_semaphore_count: 1,
            p_signal_semaphores: compute_signal_semaphores.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: compute_command_buffers.as_ptr(),
            ..Default::default()
        };

        let graphics_submit_i = vk::SubmitInfo {
            wait_semaphore_count: 2,
            p_wait_semaphores: graphics_wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: graphics_wait_stages.as_ptr(),
            signal_semaphore_count: 1,
            p_signal_semaphores: graphics_signal_semaphores.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: graphics_command_buffers.as_ptr(),
            ..Default::default()
        };

        self.device.device.queue_submit(self.device.queue_compute.0, &[compute_submit_i], vk::Fence::null()).unwrap();
        self.device.device.queue_submit(self.device.queue_graphics.0, &[graphics_submit_i], active_frame.in_flight_fence.fence).unwrap();

        let swapchains = [self.swapchain.swapchain];

        let present_i = vk::PresentInfoKHR::builder()
            .wait_semaphores(&graphics_signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&present_indices);

        self.swapchain.swapchain_init.queue_present(self.device.queue_present.0, &present_i).unwrap();
    }
}