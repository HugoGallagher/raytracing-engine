#version 450

layout(push_constant) uniform push_constants {
    mat4 view_proj;
    mat4 model;
} mats;

layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 col;

layout(location = 0) out vec3 f_col;

void main() {
    gl_Position = mats.view_proj * mats.model * vec4(pos, 1);
    f_col = (col + 1) / 2;
}