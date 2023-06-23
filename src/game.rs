use crate::math::vec::Vec2;
use crate::window::Window;

use crate::renderer::Renderer;
use crate::frametime::Frametime;

use std::collections::HashMap;
use winit::event::{VirtualKeyCode, ElementState};

pub struct Game {
    pub renderer: Renderer,

    pub keys: HashMap<VirtualKeyCode, ElementState>,
    pub screen_res: Vec2,

    pub frametime: Frametime,
}

impl Game {
    pub unsafe fn new(w: &Window, r: Vec2) -> Game {
        Game {
            renderer: Renderer::new(w),
            keys: HashMap::new(),
            screen_res: r,

            frametime: Frametime::new(),
        }
    }

    pub unsafe fn main_loop(&mut self) {
        self.frametime.refresh();

        self.update();
        //self.frametime.set("Game");

        self.draw();
        self.frametime.set("Draw");

        //println!("{}", self.frametime);
    }

    pub fn update(&mut self) {
        
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