#version 450

layout(location = 0) in vec3 Vertex_Position;

layout(set = 2, binding = 5) uniform texture2D MapMaterial_heightmap_texture;
layout(set = 2, binding = 6) uniform sampler MapMaterial_heightmap_texture_sampler;

layout(set = 0, binding = 0) uniform Camera {
    mat4 ViewProj;
};

layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};

const vec3 LIGHT_VECTOR = normalize(vec3(0.2, 1.0, 0.7));

void main() {
    vec3 transformed_position = (Model * vec4(Vertex_Position, 1.0)).xyz;
    gl_Position = ViewProj * vec4(transformed_position, 1.0);
}
