#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(binding = 0) uniform UBO {
    vec4 col;
} ubo;

layout(binding = 1) readonly buffer SSBO {
    vec4 cols[];
} ssbo;

layout(location = 0) out vec4 outColor;

void main() {
    outColor = ssbo.cols[2];
}