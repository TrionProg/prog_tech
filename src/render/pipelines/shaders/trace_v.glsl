#version 150 core
layout(std140) uniform c_globals {
    mat4 u_proj_view_matrix;
};

uniform mat4 u_model_matrix;
in vec3 a_pos;

void main() {
    gl_Position = (u_proj_view_matrix * u_model_matrix) * vec4(a_pos, 1.0);
}
