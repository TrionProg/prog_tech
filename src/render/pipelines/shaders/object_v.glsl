#version 150 core
layout(std140) uniform c_globals {
    mat4 u_proj_view_matrix;
};

uniform mat4 u_model_matrix;
in vec3 a_pos;
in vec2 a_uv;

out vec2 v_uv;

void main() {
    v_uv = a_uv;
    gl_Position = (u_proj_view_matrix * u_model_matrix) * vec4(a_pos, 1.0);
}
