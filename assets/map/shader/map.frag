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

vec4 sample_grass(vec2 coord) {
    return texture(sampler2D(MapMaterial_forest, MapMaterial_forest_sampler), coord * 0.005);
}

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

ivec2 closest_corner(vec2 pos, ivec2 corners[4]) {
    ivec2 closest_pos;
    float closest_dist = 1000000.0;

    for (int i = 0; i < 4; i++) {
        ivec2 corner_pos = corners[i];
        float dist = distance(pos, corner_pos);

        if (dist < closest_dist) {
            closest_pos = corner_pos;
            closest_dist = dist;
        }
    }

    return closest_pos;
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

    bool top = is_water(ivec2(ipos.x, ipos.y - 1));
    bool left = is_water(ivec2(ipos.x - 1, ipos.y));
    bool top_left = is_water(ivec2(ipos.x - 1, ipos.y - 1));
    bool top_right = is_water(ivec2(ipos.x + 1, ipos.y - 1));
    bool bottom_left = is_water(ivec2(ipos.x - 1, ipos.y + 1));

    float fracx = fract(pos.x);
    float fracy = fract(pos.y);
    float brightness = mix(mix(centre.brightness, right.brightness, fracx), mix(bottom.brightness, bottom_right.brightness, fracx), fracy);

    vec4 color = vec4(0.0, 0.0, 0.0, 1.0);

    if (centre.is_water) {
        color = vec4(0.0, 0.0, 1.0, 1.0);
    } else if (right.is_water || left || top || bottom.is_water || top_left || top_right || bottom_left || bottom_right.is_water) {
        color = vec4(1.0, 1.0, 0.0, 1.0);
    } else {
        color = sample_grass(pos);
    }

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
