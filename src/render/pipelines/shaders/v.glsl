uniform mat4 u_final_matrix;
attribute vec3 a_pos;
attribute vec2 a_uv;

varying vec2 v_uv;

void main() {
    v_uv = a_uv;
    gl_Position = u_final_matrix * vec4(a_pos, 1.0);
}
