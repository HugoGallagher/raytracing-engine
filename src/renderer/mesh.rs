use std::fs;
use std::io::Read;

use crate::math::vec::Vec3;
use crate::math::vec::Vec4;

pub trait FromObjTri {
    fn from_obj_tri(tri: Tri) -> Self;
}

#[derive(Copy, Clone)]
pub struct Tri {
    pub verts: [Vec4; 3],
    pub normal: Vec4,
}

impl Tri {
    pub fn new(v0: Vec3, v1: Vec3, v2: Vec3) -> Tri {
        let normal = Vec4::from_vec3(Vec3::cross(v1 - v0, v2 - v0).normalize());
        
        Tri {
            verts: [Vec4::from_vec3(v0), Vec4::from_vec3(v1), Vec4::from_vec3(v2)],
            normal: normal,
        }
    }
}

enum ObjParserState {
    Inactive,
    Verts(usize, usize),
    Faces(usize, usize),
}

pub fn parse_obj_as_tris<T: FromObjTri>(tris: &mut Vec<T>, name: &str) {
    let mut file = fs::File::open(name).unwrap();
    let mut raw = String::new();
    file.read_to_string(&mut raw).unwrap();

    let mut state = ObjParserState::Inactive;

    let mut vs: Vec<[f32; 3]> = vec![];
    let mut fs: Vec<[[f32; 3]; 3]> = Vec::new();

    let mut i: usize = 0;
    let mut c_prev: char = 0 as char;
    for c in raw.chars() {
        match state {
            ObjParserState::Inactive => {
                if c == 'v' && c_prev == 10 as char {
                    vs.push([0.0, 0.0, 0.0]);

                    state = ObjParserState::Verts(0, 0);
                }
                if c == 'f' && c_prev == 10 as char {
                    fs.push([[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]]);

                    state = ObjParserState::Faces(0, 0);
                }
            }
            ObjParserState::Verts(count, start) => {
                if char::is_whitespace(c) {
                    if start != 0 {
                        let v = vs.last_mut().unwrap();

                        v[count] = raw[start..i].parse::<f32>().unwrap();

                        if count == 2 {
                            state = ObjParserState::Inactive;
                        } else {
                            state = ObjParserState::Verts(count + 1, i + 1);
                        }
                    } else {
                        state = ObjParserState::Verts(count, i + 1);
                    }
                }
            }
            ObjParserState::Faces(count, start) => {
                if char::is_whitespace(c) {
                    if start != 0 {
                        let f = fs.last_mut().unwrap();
                        let vi = raw[start..i].parse::<usize>().unwrap();

                        f[count] = vs[vi - 1];

                        if count == 2 {
                            state = ObjParserState::Inactive;
                        } else {
                            state = ObjParserState::Faces(count + 1, i + 1);
                        }
                    } else {
                        state = ObjParserState::Faces(count, i + 1);
                    }
                }
            }
        }
        i += 1;
        c_prev = c;
    }

    for i in &fs {
        let tri = Tri::new(Vec3::new(i[0][0], i[0][1], i[0][2]), Vec3::new(i[1][0], i[1][1], i[1][2]), Vec3::new(i[2][0], i[2][1], i[2][2]));
        let formatted_tri = T::from_obj_tri(tri.clone());

        tris.push(formatted_tri);
    }
}