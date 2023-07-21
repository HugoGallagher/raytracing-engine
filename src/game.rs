use std::{sync::mpsc, f32::consts::PI};
use std::thread;

use crate::{math::{vec::{Vec2, Vec3, Vec4}, mat::Mat4, quat::Quat}, renderer::{vertex_buffer::{VertexAttribute, VertexAttributes, NoVertices}, mesh::{FromObjTri, self}, graphics_pass::{GraphicsPassDrawInfo, GraphicsPassBuilder}, push_constant::PushConstantBuilder, buffer::BufferBuilder, image::{ImageBuilder, self}, descriptors::{image_descriptor::ImageDescriptorBuilder, sampler_descriptor::SamplerDescriptorBuilder, storage_descriptor::StorageDescriptorBuilder, DescriptorsBuilder}, compute_pass::{ComputePassDispatchInfo, ComputePassBuilder}}};

use crate::renderer::Renderer;
use crate::frametime::Frametime;

use std::collections::HashMap;
use ash::vk;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};
use winit::{event::{VirtualKeyCode, ElementState}, window::Window};

#[repr(C)]
pub struct RaytracerPushConstant {
    pub view: Mat4,
    pub pos: Vec3,
    pub downscale: u32,
    pub tri_count: u32,
}

#[repr(C, align(16))]
pub struct RaytracerTri {
    pub verts: [Vec4; 3],
    pub normal: Vec4,
    pub col: Vec4,
}

impl FromObjTri for RaytracerTri {
    fn from_obj_tri(tri: mesh::Tri) -> RaytracerTri {
        RaytracerTri {
            verts: tri.verts.clone(),
            normal: tri.normal,
            col: tri.normal,
        }
    }
}

#[repr(C)]
pub struct MeshPushConstant {
    pub view_proj: Mat4,
    pub model: Mat4,
}

#[repr(C)]
pub struct MeshVertex {
    pub pos: Vec3,
    pub col: Vec3,
}

impl VertexAttributes for MeshVertex {
    fn get_attribute_data() -> Vec<VertexAttribute> {
        vec![
            VertexAttribute { format: vk::Format::R32G32B32_SFLOAT, offset: 0 },
            VertexAttribute { format: vk::Format::R32G32B32_SFLOAT, offset: 12 },
        ]
    }
}

pub struct Game {
    pub renderer: Renderer,

    pub keys: HashMap<VirtualKeyCode, ElementState>,
    pub screen_res: Vec2,

    pub frametime: Frametime,

    pub mouse_delta: Vec2,

    sens: f32,

    pos: Vec3,
    vel: Vec3,
    rot: Vec3,

    raytracer_push_constant: RaytracerPushConstant,
    mesh_push_constant: MeshPushConstant,
    
    tris: Vec<RaytracerTri>,
}

impl Game {
    pub unsafe fn new(window: RawWindowHandle, display: RawDisplayHandle, r: Vec2) -> Game {
        const MAX_TRIS: usize = 8192;
        const DOWNSCALE: u32 = 2;

        let mut tris = Vec::<RaytracerTri>::with_capacity(MAX_TRIS);

        mesh::parse_obj_as_tris::<RaytracerTri>(&mut tris, "res/meshes/asdf.obj");

        let raytracer_push_constant = RaytracerPushConstant {
            view: Mat4::identity(),
            pos: Vec3::zero(),
            downscale: DOWNSCALE,
            tri_count: tris.len() as u32,
        };
        
        let mut game = Game {
            renderer: Renderer::new(window, display),
            keys: HashMap::new(),
            screen_res: r,

            frametime: Frametime::new(),

            mouse_delta: Vec2::new(0.0, 0.0),

            sens: 0.001,

            pos: Vec3::new(0.0, 0.0, -3.0),
            vel: Vec3::new(0.0, 0.0, 0.0),
            rot: Vec3::new(0.0, 0.0, 0.0),

            raytracer_push_constant,
            mesh_push_constant: MeshPushConstant { view_proj: Mat4::identity(), model: Mat4::identity() },

            tris,
        };

        let buffer_builder = BufferBuilder::new()
            .size(std::mem::size_of::<RaytracerTri>() * MAX_TRIS)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .usage(vk::BufferUsageFlags::STORAGE_BUFFER)
            .properties(vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);

        let image_builder = ImageBuilder::new()
            .width(1280 / DOWNSCALE)
            .height(720 / DOWNSCALE)
            .usage(vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED)
            .format(vk::Format::R8G8B8A8_UNORM);

        game.renderer.add_buffer("tris", buffer_builder);
        game.renderer.add_image("raytraced_image", image_builder);

        let image_descriptor_builder = ImageDescriptorBuilder::new()
            .images(&game.renderer.images.get("raytraced_image").unwrap());
            
        let sampler_descriptor_builder = SamplerDescriptorBuilder::new()
            .images(&game.renderer.images.get("raytraced_image").unwrap());

        let storage_descriptor_builder = StorageDescriptorBuilder::new()
            .buffers(&game.renderer.buffers.get("tris").unwrap());

        let raytracer_descriptors_builder = DescriptorsBuilder::new()
            .count(game.renderer.frames_in_flight)
            .stage(vk::ShaderStageFlags::COMPUTE)
            .add_storage_builder(storage_descriptor_builder)
            .add_image_builder(image_descriptor_builder);

        let raytracer_push_constant_builder = PushConstantBuilder::new()
            .size(std::mem::size_of::<RaytracerPushConstant>())
            .stage(vk::ShaderStageFlags::COMPUTE);

        let raytracer_pass_dispatch_info = ComputePassDispatchInfo {
            x: (1280 / 16) / DOWNSCALE + 1,
            y: (720 / 16) / DOWNSCALE + 1,
            z: 1
        };

        let raytracer_pass_builder = ComputePassBuilder::new()
            .compute_shader("raytracer.comp")
            .descriptors_builder(raytracer_descriptors_builder)
            .push_constant_builder(raytracer_push_constant_builder)
            .dispatch_info(raytracer_pass_dispatch_info);

        let quad_pass_descriptors_builder = DescriptorsBuilder::new()
            .count(game.renderer.frames_in_flight)
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .add_sampler_builder(sampler_descriptor_builder);

        let quad_pass_draw_info = GraphicsPassDrawInfo::simple_vertex(6);

        let quad_pass_builder = GraphicsPassBuilder::<NoVertices>::new()
            .vertex_shader("draw_to_screen.vert")
            .fragment_shader("draw_to_screen.frag")
            .draw_info(quad_pass_draw_info)
            .targets(&game.renderer.swapchain.images)
            .descriptors_builder(quad_pass_descriptors_builder);

        let mesh_push_constant_builder = PushConstantBuilder::new()
            .size(std::mem::size_of::<MeshPushConstant>())
            .stage(vk::ShaderStageFlags::VERTEX);

        let mut mesh_tris = Vec::<RaytracerTri>::with_capacity(2048);
        mesh::parse_obj_as_tris(&mut mesh_tris, "res/meshes/torus.obj");

        let mut mesh_verts = Vec::<MeshVertex>::with_capacity(2048);
        for tri in mesh_tris {
            mesh_verts.push(MeshVertex { pos: tri.verts[0].to_vec3(), col: tri.normal.to_vec3() });
            mesh_verts.push(MeshVertex { pos: tri.verts[1].to_vec3(), col: tri.normal.to_vec3() });
            mesh_verts.push(MeshVertex { pos: tri.verts[2].to_vec3(), col: tri.normal.to_vec3() });
        }
        
        let mesh_pass_draw_info = GraphicsPassDrawInfo::simple_vertex(mesh_verts.len());

        let mesh_pass_builder = GraphicsPassBuilder::new()
            .vertex_shader("mesh.vert")
            .fragment_shader("mesh.frag")
            .draw_info(mesh_pass_draw_info)
            .targets(&game.renderer.swapchain.images)
            .extent(vk::Extent2D { width: 320, height: 180 })
            .verts(&mesh_verts)
            .push_constant_builder(mesh_push_constant_builder)
            .with_depth_buffer();

        game.renderer.add_graphics_layer("final_layer", true);
        game.renderer.add_graphics_pass("final_layer", "draw_image_to_screen", quad_pass_builder);
        //game.renderer.add_graphics_pass("final_layer", "mesh_draw", mesh_pass_builder);

        game.renderer.add_compute_layer("raytracer_layer");
        game.renderer.add_compute_pass("raytracer_layer", "raytracer", raytracer_pass_builder);

        game
    }

    pub unsafe fn main_loop(&mut self) {
        self.frametime.refresh();

        self.update();
        self.frametime.set("Game");

        self.draw();
        self.frametime.set("Draw");
    }

    pub fn update(&mut self) {
        self.rot.x -= self.mouse_delta.y * self.sens;
        self.rot.y += self.mouse_delta.x * self.sens;

        if self.key_down(VirtualKeyCode::W) {
            self.vel.z = -0.2;
        }
        if self.key_down(VirtualKeyCode::S) {
            self.vel.z = 0.2;
        }
        if self.key_down(VirtualKeyCode::A) {
            self.vel.x = -0.2;
        }
        if self.key_down(VirtualKeyCode::D) {
            self.vel.x = 0.2;
        }

        if self.key_down(VirtualKeyCode::Space) {
            self.vel.y = 0.2;
        }
        if self.key_down(VirtualKeyCode::LShift) {
            self.vel.y = -0.2;
        }

        self.pos.x += self.vel.x * self.rot.y.cos() - self.vel.z * self.rot.y.sin();
        self.pos.z -= self.vel.x * self.rot.y.sin() + self.vel.z * self.rot.y.cos();
        self.pos.y += self.vel.y;

        self.raytracer_push_constant.view = Mat4::rot_y(self.rot.y) * Mat4::rot_x(self.rot.x);
        self.raytracer_push_constant.pos = Vec3::new(self.pos.x, self.pos.y, self.pos.z);

        let mut view_dir = Vec3::new(0.0, 0.0, 1.0);

        view_dir.x = self.rot.x.cos() * self.rot.y.sin();
        view_dir.y = self.rot.x.sin();
        view_dir.z = self.rot.x.cos() * self.rot.y.cos();
        
        self.mesh_push_constant.view_proj = (Mat4::view(view_dir, self.pos) * Mat4::perspective(16.0 / 9.0, PI / 2.0, 0.0005, 100.0)).transpose();
        //self.renderer.mesh_push_constant.view_proj = (Mat4::view(view_dir, self.pos) * Mat4::orthogonal(16.0 / 9.0, 0.5, 0.0005, 100.0)).transpose();
        
        self.vel = Vec3::new(0.0, 0.0, 0.0);
        self.mouse_delta = Vec2::new(0.0, 0.0);
    }

    pub unsafe fn draw(&mut self) {
        self.renderer.pre_draw();

        self.renderer.fill_push_constant("raytracer_layer", "raytracer", &self.raytracer_push_constant);
        //self.renderer.fill_push_constant("final_layer", "mesh_draw", &self.mesh_push_constant);
        self.renderer.fill_buffer("tris", &self.tris);

        self.renderer.draw();
    }

    pub fn update_key(&mut self, vk: VirtualKeyCode, s: ElementState) {
        self.keys.insert(vk, s);
    }

    fn key_down(&self, vk: VirtualKeyCode) -> bool {
        match self.keys.get(&vk).unwrap_or(&ElementState::Released) {
            &ElementState::Pressed => true,
            &ElementState::Released => false,
        }
    }
}