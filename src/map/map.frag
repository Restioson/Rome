#version 450

layout(location = 0) out vec4 o_Target;

layout(location = 0) in vec2 v_uvs;
layout(location = 1) in float v_brightness;
layout(location = 2) in vec2 v_position;

layout(set = 1, binding = 1) uniform texture2D MapMaterial_texture;
layout(set = 1, binding = 2) uniform sampler MapMaterial_texture_sampler;
layout(set = 2, binding = 1) uniform TimeNode_time {
    float time;
};

void main() {
    vec4 color;

    if (v_uvs[0] < 1.0) { // Land
        color = texture(sampler2D(MapMaterial_texture, MapMaterial_texture_sampler), v_position.xy * 0.1);
    } else if (v_uvs[0] < 2.0) { // Beach
        color = vec4(1.0, 1.0, 0.0, 1.0);
    } else { // Ocean
        color = vec4(0.0, 0.0, (cos(time / 2.0) + 1.0) / 2.0, 1.0);
    }

    o_Target = vec4(color.xyz * v_brightness, 1.0);
}
