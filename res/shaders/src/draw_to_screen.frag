#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(set = 0, binding = 0) uniform sampler2D img;

layout(location = 0) in vec2 coord;

layout(location = 0) out vec4 outColor;

void main() {
    outColor = texture(img, coord);
}