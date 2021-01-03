#version 450

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) out vec3 v_position;

layout(set = 2, binding = 4) uniform itexture2D MapMaterial_heightmap;
layout(set = 2, binding = 5) uniform sampler MapMaterial_heightmap_sampler;

layout(set = 0, binding = 0) uniform Camera {
    mat4 ViewProj;
};

layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};

void main() {
    v_position = Vertex_Position;
    float height = texelFetch(isampler2D(MapMaterial_heightmap, MapMaterial_heightmap_sampler), ivec2(v_position.xz), 0).r * 0.01;
    v_position.y = height;

    gl_Position = ViewProj * Model * vec4(v_position, 1.0);
}
