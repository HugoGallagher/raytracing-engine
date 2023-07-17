use std::f32::consts::PI;

use crate::math::vec::{Vec2, Vec3};
use crate::renderer::vertex_buffer::{VertexAttribute, VertexAttributes};

pub struct WireframeMesh {
    pub verts: Vec<WireframeVert>,
}

impl WireframeMesh {
    pub fn blank() -> WireframeMesh {
        WireframeMesh { verts:  vec![] }
    }

    pub fn new() -> WireframeMesh {
        let mut mesh = WireframeMesh::blank();

        let u_freq = 10;
        let v_freq = 20;
        let r = 5.0;

        mesh.verts.resize(u_freq * v_freq, WireframeVert::new());

        for vi in 0..v_freq {
            for ui in 0..u_freq {
                let i = vi * u_freq + ui;

                let u = (ui as f32 / u_freq as f32) * PI * 2.0;
                let v = (vi as f32 / v_freq as f32) * PI * 2.0;

                mesh.verts[i].pos = Vec3::new((r + v.cos()) * u.sin(), (r + v.cos()) * u.cos(), v.cos());
            }
        }

        mesh
    }    
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct WireframeVert {
    pub pos: Vec3,
}

impl WireframeVert {
    pub fn new() -> WireframeVert {
        WireframeVert { pos: Vec3::zero() }
    }
}

impl VertexAttributes for WireframeVert {
    fn get_attribute_data() -> Vec<VertexAttribute> {
        vec![
            VertexAttribute {
                format: ash::vk::Format::R32G32B32_SFLOAT,
                offset: 0,
            }
        ]
    }
}