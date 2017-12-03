uniform vec4 u_basic_color;
uniform sampler2D t_tex;
varying vec2 v_uv;

void main() {
    gl_FragColor = u_basic_color * texture2D(t_tex, v_uv);
    //gl_FragColor = u_basic_color * v_uv.x;
}
