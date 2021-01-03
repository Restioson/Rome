#version 450
layout(location = 0) out vec4 o_Target;
layout(location = 1) in vec3 v_position;

layout(set = 2, binding = 0) uniform texture2D MapMaterial_forest;
layout(set = 2, binding = 1) uniform sampler MapMaterial_forest_sampler;

void main() {
    o_Target = vec4(vec3(1.0, 0.0, 1.0) * v_position.y, 1.0);
}