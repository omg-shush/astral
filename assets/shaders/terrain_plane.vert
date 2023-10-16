#version 450

layout(location = 0) in vec3 vertex_position;
layout(location = 1) in vec3 vertex_normal;
// layout(location = 2) in vec2 vertex_uv;

layout(location = 1) out vec3 out_vertex_normal;
layout(location = 2) out vec3 out_vertex_position_world;

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
    mat4 View;
    mat4 InverseView;
    mat4 Projection;
    vec3 WorldPosition;
    float width;
    float height;
};

layout(set = 2, binding = 0) uniform Mesh {
    mat4 Model;
    mat4 InverseTransposeModel;
    uint flags;
};

void main() {
    mat4 mvp = ViewProj * Model;
    gl_Position = mvp * vec4(vertex_position, 1.0);
    out_vertex_normal = vertex_normal;
    out_vertex_position_world = (Model * vec4(vertex_position, 1.0)).xyz;
}
