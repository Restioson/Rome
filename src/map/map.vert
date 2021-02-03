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

const float F = 1.0 / 8.0;
const uint HEIGHT_BITS = 11;
const uint LIGHT_BITS = 16 - HEIGHT_BITS;

vec2 round_to_increment(vec2 value, float increment) {
    return round(value * (1.0 / increment)) * increment;
}

void main() {
    lod = int(Vertex_Position.y);
    float snap_to_increment = 1.0 * (1 << lod);

    vec3 camera_pos = Model[3].xyz;
    vec2 object_to_world = round_to_increment(camera_pos.xz, snap_to_increment);
    world_space_position = vec3(Vertex_Position.x + object_to_world.x, 0.0, Vertex_Position.z + object_to_world.y);
    vec2 object_to_world_transformed = round_to_increment(camera_pos.xz * F, F * snap_to_increment);
    vec3 transformed_pos = vec3(Vertex_Position.x * F + object_to_world_transformed.x, 0.0, Vertex_Position.z * F + object_to_world_transformed.y);

    uint packed = texelFetch(usampler2D(MapMaterial_heightmap, MapMaterial_heightmap_sampler), ivec2(world_space_position.xz), 0).r;
    uint height = packed >> LIGHT_BITS;

    float y = height * 0.01 * F;
    transformed_pos.y = y;
    gl_Position = ViewProj * vec4(transformed_pos, 1.0);
}

//void main() {
//    lod = int(Vertex_Position.y);
//
//    vec3 camera_pos = Model[3].xyz;
//    ivec2 texel_coord = ivec2(floor((Vertex_Position.xz + camera_pos.xz) * F));
//    world_space_position = vec3(texel_coord.x, 0.0, texel_coord.y);
//
//    uint packed = texelFetch(usampler2D(MapMaterial_heightmap, MapMaterial_heightmap_sampler), texel_coord, 0).r;
//    uint height = packed >> LIGHT_BITS;
//
//    float y = height * 0.01;
//    world_space_position.y = y;
//    gl_Position = ViewProj * vec4(world_space_position, 1.0);
//}
