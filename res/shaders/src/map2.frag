#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(push_constant) uniform push_constants {
    vec2 pos;
} scene;

layout(location = 0) in vec2 coord;

layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;

const float SIZE = 0.05;

const float NUM_GRID_LINES = 10.0;
const float GRID_LINE_GAP = 1.0 / NUM_GRID_LINES;
const float GRID_LINE_THICKNESS = 0.05 / NUM_GRID_LINES;

void main() {
    float dst_to_pos = distance(coord, scene.pos);

    vec2 grid_line_indices;
    grid_line_indices.x = floor(coord.x / GRID_LINE_GAP);
    grid_line_indices.y = floor(coord.y / GRID_LINE_GAP);

    vec2 grid_line_coords = grid_line_indices / NUM_GRID_LINES;

    bool grid_line_horizontal = abs(coord.x - grid_line_coords.x) < GRID_LINE_THICKNESS;
    bool grid_line_vertical = abs(coord.y - grid_line_coords.y) < GRID_LINE_THICKNESS;

    bool within_circle = dst_to_pos < SIZE;
    bool grid_line = grid_line_horizontal || grid_line_vertical;

    vec4 color;
    if (within_circle) {
        color = vec4(1.0, 0.0, 0.5, 1.0);
    } else if (grid_line) {
        color = vec4(0.5, 0.0, 1.0, 1.0);
    } else {
        color = vec4(0.9, 0.8, 1.0, 1.0);
    }

    imageStore(img, ivec2(coord * 512), color);

    outColor = vec4(1.0, 0.0, 0.0, 1.0);
}