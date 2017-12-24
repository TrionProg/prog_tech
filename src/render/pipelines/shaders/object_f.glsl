#version 150 core

in vec2 v_uv;
out vec4 Target0;

uniform sampler2D t_texture;

void main() {
    Target0 = texture(t_texture, v_uv);
}
