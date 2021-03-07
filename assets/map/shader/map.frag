#version 450
layout(location = 0) out vec4 o_Target;
layout(location = 1) in vec3 world_space_position;
layout(location = 2) flat in int lod;

layout(set = 2, binding = 0) uniform texture2D MapMaterial_forest;
layout(set = 2, binding = 1) uniform sampler MapMaterial_forest_sampler;
layout(set = 2, binding = 2) uniform texture2D MapMaterial_sand;
layout(set = 2, binding = 3) uniform sampler MapMaterial_sand_sampler;
layout(set = 2, binding = 4) uniform utexture2D MapMaterial_heightmap;
layout(set = 2, binding = 5) uniform sampler MapMaterial_heightmap_sampler;

layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};

const uint LIGHT_BITS = 8;
const float MAX_LIGHT_LEVEL = float((1 << LIGHT_BITS) - 1);
const float SQRT_2 = sqrt(2.0);

struct HeightmapTexel {
    bool is_water;
    float brightness;
};

HeightmapTexel sample_heightmap(ivec2 pos) {
    uvec4 packed = texelFetch(usampler2D(MapMaterial_heightmap, MapMaterial_heightmap_sampler), pos, 0);
    return HeightmapTexel(packed.b == 1, float(packed.g) / MAX_LIGHT_LEVEL);
}

float w(HeightmapTexel t) {
    return float(t.is_water);
}

vec4 sample_raw_terrain_color(ivec2 heightmap_coord, vec2 texture_coord) {
    uvec4 packed = texelFetch(usampler2D(MapMaterial_heightmap, MapMaterial_heightmap_sampler), heightmap_coord, 0);
    uint terrain = packed.b;
    float brightness = float(packed.g) / MAX_LIGHT_LEVEL;

    vec4 color;

    if (terrain == 0) {
        color = texture(sampler2D(MapMaterial_forest, MapMaterial_forest_sampler), texture_coord * 0.005);
    } else if (terrain == 1) {
        color = vec4(0.0, 0.0, 1.0, 1.0);
    } else if (terrain == 2) {
        color = vec4(1.0, 1.0, 0.0, 1.0);
    }

    color.rgb *= brightness;
    return color;
}

vec4 f(vec2 dest, vec2 ipos) {
    return sample_raw_terrain_color(ivec2(ipos), dest);
}

vec4 sample_billinear_terrain_color(vec2 dest_coord) {
    vec2 fxy = fract(dest_coord);
    ivec2 ipos = ivec2(floor(dest_coord));

    vec4 centre = sample_raw_terrain_color(ipos, dest_coord);
    vec4 bottom = sample_raw_terrain_color(ivec2(ipos.x, ipos.y + 1), dest_coord);
    vec4 right = sample_raw_terrain_color(ivec2(ipos.x + 1, ipos.y), dest_coord);
    vec4 bottom_right = sample_raw_terrain_color(ivec2(ipos.x + 1, ipos.y + 1), dest_coord);

    return mix(mix(centre, right, fxy.x), mix(bottom, bottom_right, fxy.x), fxy.y);
}

void main() {
    o_Target = sample_billinear_terrain_color(world_space_position.xz);

//    vec4 lod_color;
//
//    if (lod < 2) {
//        lod_color = color;
//    } else if (lod == 2) {
//        lod_color = vec4(0.0, 0.0, 1.0, 1.0);
//    } else if (lod == 3) {
//        lod_color = vec4(1.0, 0.0, 1.0, 1.0);
//    } else if (lod == 4) {
//        lod_color = vec4(1.0, 0.0, 0.5, 1.0);
//    } else {
//        lod_color = vec4(0.5, 0.0, 0.5, 1.0);
//    }
//
//    o_Target = mix(color, lod_color, 0.05);
}
