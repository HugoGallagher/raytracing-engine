#version 450

layout(set = 0, binding = 0) uniform sampler2D img;

layout(location = 0) in vec2 uv;

layout(location = 0) out vec4 out_col;

void main() 
{
	out_col = texture(img, uv.yx);
}
