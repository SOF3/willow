uniform lowp float u_alpha;

varying lowp vec3 v_color;

void main() {
	gl_FragColor = vec4(v_color, u_alpha);
}
