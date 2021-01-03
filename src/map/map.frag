#version 450
layout(location = 0) out vec4 o_Target;
layout(location = 1) in vec3 v_position;

layout(set = 2, binding = 0) uniform texture2D MapMaterial_forest;
layout(set = 2, binding = 1) uniform sampler MapMaterial_forest_sampler;
layout(set = 2, binding = 2) uniform texture2D MapMaterial_sand;
layout(set = 2, binding = 3) uniform sampler MapMaterial_sand_sampler;
layout(set = 2, binding = 4) uniform itexture2D MapMaterial_heightmap;
layout(set = 2, binding = 5) uniform sampler MapMaterial_heightmap_sampler;

layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};

const vec3 LIGHT_VECTOR = normalize(vec3(1.0, 0.5, 0.3));

void main() {
    vec3 normal = normalize(vec3(dFdx(v_position.y), 1, dFdy(v_position.y)));
    vec4 transformed_normal = Model * vec4(normal, 0.0);
    float brightness = max(dot(normalize(transformed_normal.xyz), LIGHT_VECTOR), 0.0);
    vec4 color = texture(sampler2D(MapMaterial_forest, MapMaterial_forest_sampler), v_position.xz * 0.01);
    color.rgb *= brightness;

    o_Target = color;
}