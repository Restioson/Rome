#version 450

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec3 Vertex_Normal;
layout(location = 2) in vec2 Vertex_Uv;

layout(location = 0) out vec2 v_uvs;
layout(location = 1) out float v_brightness;
layout(location = 2) out vec2 v_position;

layout(set = 0, binding = 0) uniform Camera {
    mat4 ViewProj;
};

layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};

const vec3 LIGHT_VECTOR = normalize(vec3(0.2, 1.0, 0.7));

void main() {
    vec4 transformed_normal = Model * vec4(Vertex_Normal, 0.0);
    v_brightness = max(dot(normalize(transformed_normal.xyz), LIGHT_VECTOR), 0.0);
    v_uvs = Vertex_Uv;
    v_position = Vertex_Position.xz;

    vec3 transformed_position = (Model * vec4(Vertex_Position, 1.0)).xyz;
    gl_Position = ViewProj * vec4(transformed_position, 1.0);
}
