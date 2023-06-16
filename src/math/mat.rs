use std::{ops, mem};

use crate::math::vec::{Vec2, Vec3, Vec4};
use crate::math::quat::Quat;

#[derive(Copy, Clone)]
pub struct Mat2 {
    pub x: Vec2,
    pub y: Vec2,
}

#[derive(Copy, Clone)]
pub struct Mat4 {
    pub x: Vec4,
    pub y: Vec4,
    pub z: Vec4,
    pub w: Vec4,
}

impl Mat2 {
    pub fn new(xx: f32, xy: f32, yx: f32, yy: f32) -> Mat2 {
        Mat2 {
            x: Vec2::new(xx, xy),
            y: Vec2::new(yx, yy),
        }
    }
}

impl ops::Add<Mat2> for Mat2 {
    type Output = Mat2;

    fn add(self, rhs: Mat2) -> Mat2 {
        Mat2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl ops::Sub<Mat2> for Mat2 {
    type Output = Mat2;

    fn sub(self, rhs: Mat2) -> Mat2 {
        Mat2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
impl ops::Mul<Mat2> for Mat2 {
    type Output = Mat2;

    fn mul(self, rhs: Mat2) -> Mat2 {
        Mat2 {
            x: Vec2::new(self.x.x * rhs.x.x + self.x.y * rhs.y.x, self.x.x * rhs.x.y + self.x.y * rhs.y.y),
            y: Vec2::new(self.y.x * rhs.x.x + self.y.y * rhs.y.x, self.y.x * rhs.x.y + self.y.y * rhs.y.y),
        }
    }
}

impl Mat4 {
    pub fn identity() -> Mat4 {
        Mat4 {
            x: Vec4::new(1.0, 0.0, 0.0, 0.0),
            y: Vec4::new(0.0, 1.0, 0.0, 0.0),
            z: Vec4::new(0.0, 0.0, 1.0, 0.0),
            w: Vec4::new(0.0, 0.0, 0.0, 1.0),
        }
    }

    pub fn rot(q: Quat) -> Mat4 {
        Mat4 {
            x: Vec4::new(2.0 * (q.r * q.r + q.i * q.i) - 1.0, 2.0 * (q.i * q.j - q.r * q.k), 2.0 * (q.i * q.k + q.r * q.j), 0.0),
            y: Vec4::new(2.0 * (q.i * q.j + q.r * q.k), 2.0 * (q.r * q.r + q.j * q.j) - 1.0, 2.0 * (q.j * q.k - q.r * q.i), 0.0),
            z: Vec4::new(2.0 * (q.i * q.k - q.r * q.j), 2.0 * (q.j * q.k + q.r * q.i), 2.0 * (q.r * q.r + q.k * q.k) - 1.0, 0.0),
            w: Vec4::new(0.0, 0.0, 0.0, 1.0),
        }
    }

    pub fn rot_x(r: f32) -> Mat4 {
        Mat4 {
            x: Vec4::new(1.0, 0.0, 0.0, 0.0),
            y: Vec4::new(0.0, r.cos(), -r.sin(), 0.0),
            z: Vec4::new(0.0, r.sin(), r.cos(), 0.0),
            w: Vec4::new(0.0, 0.0, 0.0, 1.0),
        }
    }
    pub fn rot_y(r: f32) -> Mat4 {
        Mat4 {
            x: Vec4::new(r.cos(), 0.0, -r.sin(), 0.0),
            y: Vec4::new(0.0, 1.0, 0.0, 0.0),
            z: Vec4::new(r.sin(), 0.0, r.cos(), 0.0),
            w: Vec4::new(0.0, 0.0, 0.0, 1.0),
        }
    }
    pub fn rot_z(r: f32) -> Mat4 {
        Mat4 {
            x: Vec4::new(r.cos(), -r.sin(), 0.0, 0.0),
            y: Vec4::new(r.sin(), r.cos(), 0.0, 0.0),
            z: Vec4::new(0.0, 0.0, 1.0, 0.0),
            w: Vec4::new(0.0, 0.0, 0.0, 1.0),
        }
    }

    pub fn transpose(&self) -> Mat4 {
        Mat4 {
            x: Vec4::new(self.x.x, self.y.x, self.z.x, self.w.x),
            y: Vec4::new(self.x.y, self.y.y, self.z.y, self.w.y),
            z: Vec4::new(self.x.z, self.y.z, self.z.z, self.w.z),
            w: Vec4::new(self.x.w, self.y.w, self.z.w, self.w.w),
        }
    }
}

impl ops::Add<Mat4> for Mat4 {
    type Output = Mat4;

    fn add(self, rhs: Mat4) -> Mat4 {
        Mat4 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: self.w + rhs.w,
        }
    }
}
impl ops::Sub<Mat4> for Mat4 {
    type Output = Mat4;

    fn sub(self, rhs: Mat4) -> Mat4 {
        Mat4 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            w: self.w - rhs.w,
        }
    }
}

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "sse"))]
impl ops::Mul<Mat4> for Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: Mat4) -> Mat4 {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::{_mm_set_ps, _mm_set_ps1, _mm_store_ps, _mm_add_ps, _mm_mul_ps};
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::{_mm_set_ps, _mm_set_ps1, _mm_store_ps, _mm_add_ps, _mm_mul_ps};

        unsafe {
            let r0 =
            _mm_add_ps(
                _mm_add_ps(
                    _mm_mul_ps(
                        _mm_set_ps1(self.x.x),
                        _mm_set_ps(rhs.x.w, rhs.x.z, rhs.x.y, rhs.x.x)
                    ),
                    _mm_mul_ps(
                        _mm_set_ps1(self.x.y),
                        _mm_set_ps(rhs.y.w, rhs.y.z, rhs.y.y, rhs.y.x),
                    )
                ),
                _mm_add_ps(
                    _mm_mul_ps(
                        _mm_set_ps1(self.x.z),
                        _mm_set_ps(rhs.z.w, rhs.z.z, rhs.z.y, rhs.z.x)
                    ),
                    _mm_mul_ps(
                        _mm_set_ps1(self.x.w),
                        _mm_set_ps(rhs.w.w, rhs.w.z, rhs.w.y, rhs.w.x),
                    )
                ),
            );
            let r1 =
            _mm_add_ps(
                _mm_add_ps(
                    _mm_mul_ps(
                        _mm_set_ps1(self.y.x),
                        _mm_set_ps(rhs.x.w, rhs.x.z, rhs.x.y, rhs.x.x)
                    ),
                    _mm_mul_ps(
                        _mm_set_ps1(self.y.y),
                        _mm_set_ps(rhs.y.w, rhs.y.z, rhs.y.y, rhs.y.x),
                    )
                ),
                _mm_add_ps(
                    _mm_mul_ps(
                        _mm_set_ps1(self.y.z),
                        _mm_set_ps(rhs.z.w, rhs.z.z, rhs.z.y, rhs.z.x)
                    ),
                    _mm_mul_ps(
                        _mm_set_ps1(self.y.w),
                        _mm_set_ps(rhs.w.w, rhs.w.z, rhs.w.y, rhs.w.x),
                    )
                ),
            );
            let r2 =
            _mm_add_ps(
                _mm_add_ps(
                    _mm_mul_ps(
                        _mm_set_ps1(self.z.x),
                        _mm_set_ps(rhs.x.w, rhs.x.z, rhs.x.y, rhs.x.x)
                    ),
                    _mm_mul_ps(
                        _mm_set_ps1(self.z.y),
                        _mm_set_ps(rhs.y.w, rhs.y.z, rhs.y.y, rhs.y.x),
                    )
                ),
                _mm_add_ps(
                    _mm_mul_ps(
                        _mm_set_ps1(self.z.z),
                        _mm_set_ps(rhs.z.w, rhs.z.z, rhs.z.y, rhs.z.x)
                    ),
                    _mm_mul_ps(
                        _mm_set_ps1(self.z.w),
                        _mm_set_ps(rhs.w.w, rhs.w.z, rhs.w.y, rhs.w.x),
                    )
                ),
            );
            let r3 =
            _mm_add_ps(
                _mm_add_ps(
                    _mm_mul_ps(
                        _mm_set_ps1(self.w.x),
                        _mm_set_ps(rhs.x.w, rhs.x.z, rhs.x.y, rhs.x.x)
                    ),
                    _mm_mul_ps(
                        _mm_set_ps1(self.w.y),
                        _mm_set_ps(rhs.y.w, rhs.y.z, rhs.y.y, rhs.y.x),
                    )
                ),
                _mm_add_ps(
                    _mm_mul_ps(
                        _mm_set_ps1(self.w.z),
                        _mm_set_ps(rhs.z.w, rhs.z.z, rhs.z.y, rhs.z.x)
                    ),
                    _mm_mul_ps(
                        _mm_set_ps1(self.w.w),
                        _mm_set_ps(rhs.w.w, rhs.w.z, rhs.w.y, rhs.w.x),
                    )
                ),
            );

            let mut r0d: [f32; 4] = [0.0, 0.0, 0.0, 0.0];
            let mut r1d: [f32; 4] = [0.0, 0.0, 0.0, 0.0];
            let mut r2d: [f32; 4] = [0.0, 0.0, 0.0, 0.0];
            let mut r3d: [f32; 4] = [0.0, 0.0, 0.0, 0.0];

            _mm_store_ps(&mut r0d[0], r0);
            _mm_store_ps(&mut r1d[0], r1);
            _mm_store_ps(&mut r2d[0], r2);
            _mm_store_ps(&mut r3d[0], r3);

            Mat4 {
                x: Vec4::new(r0d[0], r0d[1], r0d[2], r0d[3]),
                y: Vec4::new(r1d[0], r1d[1], r1d[2], r1d[3]),
                z: Vec4::new(r2d[0], r2d[1], r2d[2], r2d[3]),
                w: Vec4::new(r3d[0], r3d[1], r3d[2], r3d[3]),
            }
        }
    }
}

impl ops::Mul<Vec4> for Mat4 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Vec4 {
        Vec4 {
            x: self.x.x * rhs.x + self.x.y * rhs.y + self.x.z * rhs.z + self.x.w * rhs.w,
            y: self.y.x * rhs.x + self.y.y * rhs.y + self.y.z * rhs.z + self.y.w * rhs.w,
            z: self.z.x * rhs.x + self.z.y * rhs.y + self.z.z * rhs.z + self.z.w * rhs.w,
            w: self.w.x * rhs.x + self.w.y * rhs.y + self.w.z * rhs.z + self.w.w * rhs.w,
        }
    }
}