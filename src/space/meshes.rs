use std::f32::consts::{PI, TAU};

use ash::vk;

use crate::{math::vec::{Vec2, Vec3}, renderer::vertex_buffer::{VertexAttribute, VertexAttributes}};

pub trait SpaceMesh {
    fn get_3d_from_2d(&self, uv: Vec2) -> Vec3;
    fn gen_verts(&mut self, res: u32);
}

#[repr(C)]
pub struct SpaceVert {
    pos: Vec3,
    uv: Vec2,
}

impl VertexAttributes for SpaceVert {
    fn get_attribute_data() -> Vec<VertexAttribute> {
        vec![
            VertexAttribute { format: vk::Format::R32G32B32_SFLOAT, offset: 0 },
            VertexAttribute { format: vk::Format::R32G32_SFLOAT, offset: 12 },
        ]
    }
}

pub struct Torus {
    cs: f32,
    ring: f32,

    pub verts: Vec<SpaceVert>,
    pub indices: Vec<u32>,
}

impl SpaceMesh for Torus {
    fn get_3d_from_2d(&self, uv: Vec2) -> Vec3 {
        let scaled = Vec2::new(uv.x * TAU, uv.y * TAU);

        Vec3 {
           x: scaled.y.cos() * (self.ring + self.cs * scaled.x.cos()),
           y: self.cs * scaled.x.sin(),
           z: scaled.y.sin() * (self.ring + self.cs * scaled.x.cos()),
        }
    }

    fn gen_verts(&mut self, res: u32) {
        let cs_res = res;
        let cs_incr = 1.0 / cs_res as f32;

        let ring_res = res * (self.ring / self.cs) as u32;
        //let ring_res = res;
        let ring_incr = 1.0 / ring_res as f32;

        let total_verts = cs_res * ring_res;

        self.verts = Vec::with_capacity(total_verts as usize);
        self.indices = Vec::with_capacity((total_verts * 6) as usize);

        for i in 0..(cs_res + 1) {
            let cs_theta = i as f32 * cs_incr;

            for j in 0..(ring_res + 1) {
                let ring_theta = j as f32 * ring_incr;

                let uv = Vec2::new(cs_theta, ring_theta);
                let pos = self.get_3d_from_2d(uv);

                self.verts.push(SpaceVert {
                    pos,
                    uv,
                });
            }
        }

        for i in 0..cs_res {
            for j in 0..ring_res {
                let cs_i = i;
                let next_cs_i = (cs_i + 1) % (cs_res + 1);
                
                let ring_i = j;
                let next_ring_i = (ring_i + 1) % (ring_res + 1);

                let bottom_left = cs_i * (ring_res + 1) + ring_i;
                let bottom_right = cs_i * (ring_res + 1) + next_ring_i;

                let top_left = next_cs_i * (ring_res + 1) + ring_i;
                let top_right = next_cs_i * (ring_res + 1) + next_ring_i;

                self.indices.push(bottom_left);
                self.indices.push(bottom_right);
                self.indices.push(top_left);

                self.indices.push(top_left);
                self.indices.push(bottom_right);
                self.indices.push(top_right);
            }
        }
    }
}

impl Torus {
    pub fn new(cs: f32, ring: f32, res: u32) -> Torus {
        let verts = Vec::<SpaceVert>::new();
        let indices = Vec::<u32>::new();

        let mut torus = Torus {
            cs,
            ring,

            verts,
            indices,
        };

        torus.gen_verts(res);

        torus
    }
}