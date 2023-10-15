#version 450

layout(push_constant) uniform push_constants {
    mat4 view_proj;
    mat4 model;
} mats;

layout(location = 0) in vec3 pos;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec2 f_pos;

void main() {
    gl_Position = mats.view_proj * mats.model * vec4(pos, 1);
    f_pos = uv;
}