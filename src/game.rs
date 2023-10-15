use std::f32::consts::PI;

use crate::{math::{vec::{Vec2, Vec3, Vec4}, mat::Mat4}, renderer::{vertex_buffer::{VertexAttribute, VertexAttributes, NoVertices}, mesh::{FromObjTri, self}, graphics_pass::{GraphicsPassDrawInfo, GraphicsPassBuilder}, buffer::BufferBuilder, image::{ImageBuilder, Image}, descriptors::{CreationReference, BindingReference}, compute_pass::{ComputePassDispatchInfo, ComputePassBuilder}, layer::{LayerExecution, PassDependency}, shader::ShaderType, renderer_data::ResourceReference}, space::meshes::Torus};

use crate::renderer::Renderer;
use crate::util::frametime::Frametime;

use std::collections::HashMap;
use ash::vk;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};
use winit::event::{VirtualKeyCode, ElementState};

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
pub struct MapPushConstant {
    pub pos: Vec2,
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

    uv_pos: Vec2,

    map_push_constant: MapPushConstant,
    mesh_push_constant: MeshPushConstant,

    space_mesh: Torus,
}

impl Game {
    pub unsafe fn new(window: RawWindowHandle, display: RawDisplayHandle, r: Vec2) -> Game {
        let map_push_constant = MapPushConstant {
            pos: Vec2::zero(),
        };

        let mesh_push_constant = MeshPushConstant {
            view_proj: Mat4::identity(),
            model: Mat4::identity(),
        };

        let space_mesh = Torus::new(3.0, 10.0, 15);
        
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

            uv_pos: Vec2::zero(),

            map_push_constant,
            mesh_push_constant,

            space_mesh,
        };

        let map_image_builder = ImageBuilder::new()
            .width(1024)
            .height(1024)
            .usage(vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED)
            .format(vk::Format::B8G8R8A8_UNORM)
            .layout(vk::ImageLayout::GENERAL);

        game.renderer.add_images("map", map_image_builder);

        let map_pass_creation_refs = vec![CreationReference::Image("map".to_string())];
        let mesh_pass_creation_refs = vec![CreationReference::Sampler("map".to_string())];

        let map_pass_builder = ComputePassBuilder::new()
            .compute_shader("map.comp")
            .dispatch_info(ComputePassDispatchInfo::for_image("map", &game.renderer.data))
            .push_constant::<MapPushConstant>()
            .descriptors(map_pass_creation_refs, &game.renderer.data);

        let mesh_pass_builder = GraphicsPassBuilder::new()
            .vertex_shader("mesh.vert")
            .fragment_shader("mesh.frag")
            .draw_info(GraphicsPassDrawInfo::simple_indexed(game.space_mesh.verts.len(), game.space_mesh.indices.len()))
            .targets(&game.renderer.swapchain.images)
            .verts(&game.space_mesh.verts)
            .vertex_indices(&game.space_mesh.indices)
            .vertex_push_constant::<MeshPushConstant>()
            .fragment_descriptors(mesh_pass_creation_refs, &game.renderer.data)
            .clear_col(Vec4::new(0.82, 0.8, 0.9, 1.0))
            .with_depth_buffer();

        let pass_dependancy = PassDependency {
            resource: ResourceReference::Image(game.renderer.data.get_image_refs("map")),

            src_access: vk::AccessFlags::SHADER_WRITE,
            src_stage: vk::PipelineStageFlags::COMPUTE_SHADER,
            src_shader: ShaderType::Compute,
            
            dst_access: vk::AccessFlags::SHADER_READ,
            dst_stage: vk::PipelineStageFlags::FRAGMENT_SHADER,
            dst_shader: ShaderType::Fragment,
        };

        game.renderer.add_layer("final_layer", true, LayerExecution::Main);

        game.renderer.add_compute_pass("final_layer", "map_draw", map_pass_builder);
        game.renderer.add_graphics_pass("final_layer", "mesh_draw", mesh_pass_builder);

        game.renderer.add_pass_dependency("final_layer", "map_draw", "mesh_draw", Some(pass_dependancy));

        game.renderer.get_layer_mut("final_layer").set_root_path("mesh_draw");

        game
    }

    pub unsafe fn main_loop(&mut self) {
        let delta = self.frametime.get_delta();

        self.frametime.refresh();

        self.update(delta);
        self.frametime.set("Game");

        self.draw();
        self.frametime.set("Draw");

        //println!("{}", self.frametime);
    }

    pub fn update(&mut self, delta: f32) {
        self.rot.x -= self.mouse_delta.y * self.sens;
        self.rot.y += self.mouse_delta.x * self.sens;

        let uv_speed = 1.0;
        let uv_vel = uv_speed * delta;

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

        if self.key_down(VirtualKeyCode::I) {
            self.uv_pos.y += uv_vel;
        }
        if self.key_down(VirtualKeyCode::K) {
            self.uv_pos.y -= uv_vel;
        }
        if self.key_down(VirtualKeyCode::J) {
            self.uv_pos.x -= uv_vel;
        }
        if self.key_down(VirtualKeyCode::L) {
            self.uv_pos.x += uv_vel;
        }

        if self.uv_pos.y < 0.0 { self.uv_pos.y += 1.0 }
        if self.uv_pos.y >= 1.0 { self.uv_pos.y -= 1.0 }
        if self.uv_pos.x < 0.0 { self.uv_pos.x += 1.0 }
        if self.uv_pos.x >= 1.0 { self.uv_pos.x -= 1.0 }

        // self.uv_pos.x %= 1.0;
        // self.uv_pos.y %= 1.0;

        self.pos.x += self.vel.x * self.rot.y.cos() - self.vel.z * self.rot.y.sin();
        self.pos.z -= self.vel.x * self.rot.y.sin() + self.vel.z * self.rot.y.cos();
        self.pos.y += self.vel.y;

        let mut view_dir = Vec3::new(0.0, 0.0, 1.0);

        view_dir.x = self.rot.x.cos() * self.rot.y.sin();
        view_dir.y = self.rot.x.sin();
        view_dir.z = self.rot.x.cos() * self.rot.y.cos();
        
        self.map_push_constant.pos = self.uv_pos;
        self.mesh_push_constant.view_proj = (Mat4::view(view_dir, self.pos) * Mat4::perspective(16.0 / 9.0, PI / 2.0, 0.0005, 100.0)).transpose();
        
        self.vel = Vec3::new(0.0, 0.0, 0.0);
        self.mouse_delta = Vec2::new(0.0, 0.0);
    }

    pub unsafe fn draw(&mut self) {
        self.renderer.pre_draw();

        self.renderer.get_layer_mut("final_layer").fill_compute_push_constant("map_draw", &self.map_push_constant);
        self.renderer.get_layer_mut("final_layer").fill_vertex_push_constant("mesh_draw", &self.mesh_push_constant);

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