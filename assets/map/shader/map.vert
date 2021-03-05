#version 450

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) out vec3 world_space_position;
layout(location = 2) flat out int lod;

layout(set = 2, binding = 4) uniform utexture2D MapMaterial_heightmap;
layout(set = 2, binding = 5) uniform sampler MapMaterial_heightmap_sampler;
// TODO more mipmap levels
layout(set = 2, binding = 6) uniform utexture2D MapMaterial_mipmap;
layout(set = 2, binding = 7) uniform sampler MapMaterial_mipmap_sampler;

layout(set = 0, binding = 0) uniform Camera {
    mat4 ViewProj;
};

layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};

const float Y_SCALE = 0.2;
const float XZ_SCALE = (1.0 / 8.0);
const uint HEIGHT_BITS = 8;
const uint LIGHT_BITS = 15 - HEIGHT_BITS;
const uint LOD_0_EXTENT = 512 / 2;

void main() {
    lod = int(Vertex_Position.y);
    float grid_size = float(1 << lod);

    vec3 camera_pos = Model[3].xyz / XZ_SCALE;
    world_space_position.xz = Vertex_Position.xz + floor(camera_pos.xz / grid_size) * grid_size;

    float height;

    if (lod == 0) {
        uint packed = texelFetch(usampler2D(MapMaterial_heightmap, MapMaterial_heightmap_sampler), ivec2(world_space_position.xz), 0).r;
        float mipHeight = float(texelFetch(usampler2D(MapMaterial_mipmap, MapMaterial_mipmap_sampler), ivec2(world_space_position.xz) / 2, 0).r);
        float fullresHeight = float((packed >> LIGHT_BITS) & uint((1 << HEIGHT_BITS) - 1));
        vec2 normalizedPos = ivec2(world_space_position.xz) / LOD_0_EXTENT;
        height = mix(fullresHeight, mipHeight, min(1.0, (normalizedPos.x + normalizedPos.y) / 2));
    } else {
        height = float(texelFetch(usampler2D(MapMaterial_mipmap, MapMaterial_mipmap_sampler), ivec2(world_space_position.xz) / 2, 0).r);
    }

    vec2 transformed_pos = world_space_position.xz * XZ_SCALE;
    float y = height * Y_SCALE * XZ_SCALE;
    world_space_position.y = height;

    gl_Position = ViewProj * vec4(transformed_pos.x, y, transformed_pos.y, 1.0);
}
