#version 450

layout(location = 0) out vec4 o_Target;

layout(set = 2, binding = 1) uniform texture2D MapMaterial_forest_texture;
layout(set = 2, binding = 2) uniform sampler MapMaterial_forest_texture_sampler;
layout(set = 2, binding = 3) uniform texture2D MapMaterial_beach_texture;
layout(set = 2, binding = 4) uniform sampler MapMaterial_beach_texture_sampler;

void main() {
    vec3 color = vec3(1.0, 0.0, 1.0);
    o_Target = vec4(color, 1.0);
}
