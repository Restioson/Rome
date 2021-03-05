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

const vec3 LIGHT_VECTOR = normalize(vec3(1.0, 0.5, 0.3));
const uint HEIGHT_BITS = 8;
const uint LIGHT_BITS = 15 - HEIGHT_BITS;
const uint LIGHT_MASK = (1 << LIGHT_BITS) - 1;
const float SQRT_2 = sqrt(2.0);

struct HeightmapTexel {
    bool is_water;
    float brightness;
};

HeightmapTexel sample_heightmap(ivec2 pos) {
    uint packed = texelFetch(usampler2D(MapMaterial_heightmap, MapMaterial_heightmap_sampler), pos, 0).r;
    uint brightness_level = packed & LIGHT_MASK;
    return HeightmapTexel((packed >> 15) == 1, float(brightness_level) / float(LIGHT_MASK));
}

bool is_water(ivec2 pos) {
    uint packed = texelFetch(usampler2D(MapMaterial_heightmap, MapMaterial_heightmap_sampler), pos, 0).r;
    uint brightness_level = packed & LIGHT_MASK;
    return (packed >> 15) == 1;
}

float w(HeightmapTexel t) {
    return float(t.is_water);
}

//

vec4 sample_raw_terrain_color(ivec2 heightmap_coord, vec2 texture_coord) {
    uint packed = texelFetch(usampler2D(MapMaterial_heightmap, MapMaterial_heightmap_sampler), heightmap_coord, 0).r;
    uint brightness_level = packed & LIGHT_MASK;
    bool is_water = (packed >> 15) == 1;

    if (is_water) {
        return vec4(0.0, 0.0, 1.0, 1.0);
    } else {
        return texture(sampler2D(MapMaterial_forest, MapMaterial_forest_sampler), texture_coord * 0.005);
    }
}

// from http://www.java-gaming.org/index.php?topic=35123.0
vec4 cubic(float v){
    vec4 n = vec4(1.0, 2.0, 3.0, 4.0) - v;
    vec4 s = n * n * n;
    float x = s.x;
    float y = s.y - 4.0 * s.x;
    float z = s.z - 4.0 * s.y + 6.0 * s.x;
    float w = 6.0 - x - y - z;
    return vec4(x, y, z, w) * (1.0/6.0);
}

vec4 sample_bicubic_terrain_color(vec2 dest_coord) {
    vec2 fxy = fract(dest_coord);
    vec2 heightmap_coord = floor(dest_coord);

    vec4 xcubic = cubic(fxy.x);
    vec4 ycubic = cubic(fxy.y);

    vec4 c = heightmap_coord.xxyy + vec2 (-0.5, +1.5).xyxy;

    vec4 s = vec4(xcubic.xz + xcubic.yw, ycubic.xz + ycubic.yw);
    vec4 offset = c + vec4 (xcubic.yw, ycubic.yw) / s;

    vec4 sample0 = sample_raw_terrain_color(ivec2(offset.xz), dest_coord);
    vec4 sample1 = sample_raw_terrain_color(ivec2(offset.yz), dest_coord);
    vec4 sample2 = sample_raw_terrain_color(ivec2(offset.xw), dest_coord);
    vec4 sample3 = sample_raw_terrain_color(ivec2(offset.yw), dest_coord);

    float sx = s.x / (s.x + s.y);
    float sy = s.z / (s.z + s.w);

    return mix(mix(sample3, sample2, sx), mix(sample1, sample0, sx), sy);
}

void main() {
    vec2 pos = world_space_position.xz;
    ivec2 ipos = ivec2(floor(world_space_position.xz));

    uint packed = texelFetch(usampler2D(MapMaterial_heightmap, MapMaterial_heightmap_sampler), ipos, 0).r;
    uint brightness_level = packed & LIGHT_MASK;

    HeightmapTexel centre = sample_heightmap(ipos);
    HeightmapTexel right = sample_heightmap(ivec2(ipos.x + 1, ipos.y));
    HeightmapTexel bottom = sample_heightmap(ivec2(ipos.x, ipos.y + 1));
    HeightmapTexel bottom_right = sample_heightmap(ivec2(ipos.x + 1, ipos.y + 1));

    float fracx = fract(pos.x);
    float fracy = fract(pos.y);
    float brightness = mix(mix(centre.brightness, right.brightness, fracx), mix(bottom.brightness, bottom_right.brightness, fracx), fracy);
    vec4 color = sample_bicubic_terrain_color(pos);

    color.rgb *= brightness;

    o_Target = color;

//    vec4 lod_color;
//
//    if (lod == 0) {
//        lod_color = color;
//    } else if (lod == 1) {
//        lod_color = vec4(1.0, 1.0, 0.0, 1.0);
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
