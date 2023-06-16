use std::ops;

use crate::math::vec::{Vec3, Vec4};

#[derive(Copy, Clone)]
pub struct Quat {
    pub r: f32,
    pub i: f32,
    pub j: f32,
    pub k: f32,
}

impl Quat {
    pub fn from_a(r: f32, a: Vec3) -> Quat {
        let s = (r / 2.0).sin();
        Quat {
            r: (r / 2.0).cos(),
            i: a.x * s,
            j: a.y * s,
            k: a.z * s,
        }
    }

    pub fn from_eu(x: f32, y: f32, z: f32) -> Quat {
        let x_cos = (x / 2.0).cos();
        let x_sin = (x / 2.0).sin();
        let y_cos = (y / 2.0).cos();
        let y_sin = (y / 2.0).sin();
        let z_cos = (z / 2.0).cos();
        let z_sin = (z / 2.0).sin();
        
        Quat {
            r: x_cos * y_cos * z_cos + x_sin * y_sin * z_sin,
            i: x_sin * y_cos * z_cos - x_cos * y_sin * z_sin,
            j: x_cos * y_sin * z_cos + x_sin * y_cos * z_sin,
            k: x_cos * y_cos * z_sin - x_sin * y_sin * z_cos,
        }
    }

    pub fn conj(&self) -> Quat {
        Quat {
            r:  self.r,
            i: -self.i,
            j: -self.j,
            k: -self.k,
        }
    }
}

impl ops::Mul<Quat> for Quat {
    type Output = Quat;

    fn mul(self, rhs: Quat) -> Quat {
        Quat {
            r: self.r * rhs.r - self.i * rhs.i - self.j * rhs.j - self.k * rhs.j,
            i: self.r * rhs.i + self.i * rhs.r - self.j * rhs.k + self.k * rhs.j,
            j: self.r * rhs.j + self.i * rhs.k + self.j * rhs.r - self.k * rhs.i,
            k: self.r * rhs.k - self.i * rhs.j + self.j * rhs.i + self.k * rhs.r,
        }
    }
}