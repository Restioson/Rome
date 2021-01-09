#version 450

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) out vec3 world_space_position;
layout(location = 2) flat out int lod;

layout(set = 2, binding = 4) uniform utexture2D MapMaterial_heightmap;
layout(set = 2, binding = 5) uniform sampler MapMaterial_heightmap_sampler;

layout(set = 0, binding = 0) uniform Camera {
    mat4 ViewProj;
};

layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};

vec2 roundToIncrement(vec2 value, float increment) {
    return round(value * (1.0 / increment)) * increment;
}

void main() {
    lod = int(Vertex_Position.y);
    float snapToIncrement = 1.0 * (1 << lod);

    vec3 camera_pos = Model[3].xyz;
    vec2 object_to_world = roundToIncrement(camera_pos.xz, snapToIncrement);
    world_space_position = vec3(Vertex_Position.x + object_to_world.x, 0.0, Vertex_Position.z + object_to_world.y);

    uint packed = texelFetch(usampler2D(MapMaterial_heightmap, MapMaterial_heightmap_sampler), ivec2(world_space_position.xz), 0).r;
    uint height = packed >> 8;

    float y = height * 0.1;
    world_space_position.y = y;
    gl_Position = ViewProj * vec4(world_space_position, 1.0);
}
