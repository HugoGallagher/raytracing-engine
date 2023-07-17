pub mod wireframe_mesh;

use std::sync::mpsc;
use std::thread;

use crate::math::{vec::{Vec2, Vec3, Vec4}, mat::Mat4, quat::Quat};

use crate::renderer::Renderer;
use crate::frametime::Frametime;

use std::collections::HashMap;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};
use winit::{event::{VirtualKeyCode, ElementState}, window::Window};

pub struct Game {
    pub renderer: Renderer,

    pub keys: HashMap<VirtualKeyCode, ElementState>,
    pub screen_res: Vec2,

    pub frametime: Frametime,

    pub mouse_delta: Vec2,
    pub mid_mouse_pos: Vec2,

    pos: Vec3,
    vel: Vec3,
    rot: Vec3,

    sens: f32,
}

impl Game {
    pub unsafe fn new(window: RawWindowHandle, display: RawDisplayHandle, r: Vec2) -> Game {
        Game {
            renderer: Renderer::new(window, display),
            keys: HashMap::new(),
            screen_res: r,

            frametime: Frametime::new(),

            mouse_delta: Vec2::new(0.0, 0.0),
            mid_mouse_pos: Vec2::new((r.x / 2.0) as f32, (r.y / 2.0) as f32),

            pos: Vec3::new(0.0, 0.0, 0.0),
            vel: Vec3::new(0.0, 0.0, 0.0),
            rot: Vec3::new(0.0, 0.0, 0.0),

            sens: 0.001,
        }
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

        //self.renderer.push_constant.view = Mat4::rot(Quat::from_eu(self.rot.x, self.rot.y, self.rot.z));
        self.renderer.push_constant.view = Mat4::rot_x(self.rot.x) * Mat4::rot_y(self.rot.y);

        self.pos.x += self.vel.x * self.rot.y.cos() - self.vel.z * self.rot.y.sin();
        self.pos.z -= self.vel.x * self.rot.y.sin() + self.vel.z * self.rot.y.cos();
        self.pos.y += self.vel.y;

        self.renderer.push_constant.pos = Vec3::new(self.pos.x, self.pos.y, self.pos.z);

        self.vel = Vec3::new(0.0, 0.0, 0.0);
        self.mouse_delta = Vec2::new(0.0, 0.0);
    }

    pub unsafe fn draw(&mut self) {
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