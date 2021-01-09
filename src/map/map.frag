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

vec3 sample_grass(vec2 coord) {
    return texture(sampler2D(MapMaterial_forest, MapMaterial_forest_sampler), coord * 0.05).rgb;
}

void main() {
    uint packed = texelFetch(usampler2D(MapMaterial_heightmap, MapMaterial_heightmap_sampler), ivec2(world_space_position.xz), 0).r;
    uint brightness_level = packed & 0xFF;
    float brightness = float(brightness_level) / 255.0;

    vec3 color = sample_grass(world_space_position.xz);
    color.rgb *= brightness;

    vec4 lod_color;

    if (lod == 0) {
        lod_color = vec4(1.0, 0.0, 0.0, 1.0);
    } else if (lod == 1) {
        lod_color = vec4(1.0, 1.0, 0.0, 1.0);
    } else if (lod == 2) {
        lod_color = vec4(0.0, 0.0, 1.0, 1.0);
    } else if (lod == 3) {
        lod_color = vec4(1.0, 0.0, 1.0, 1.0);
    }

//    o_Target = mix(vec4(color, 1.0), lod_color, 0.1);
    o_Target = vec4(color, 1.0);
}