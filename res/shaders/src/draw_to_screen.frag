#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(binding = 0) uniform UBO {
    vec4 col;
} ubo;

layout(location = 0) out vec4 outColor;

void main() {
    outColor = ubo.col;
}