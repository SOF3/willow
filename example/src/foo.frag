uniform lowp float u_alpha;

varying lowp vec3 v_color;
varying mediump vec3 v_normal;

void main() {
	mediump vec3 normal = normalize(v_normal);
	lowp float lighting = dot(normal, vec3(1, 0, 0));
	lighting = max(lighting, 1.0);

	gl_FragColor = vec4(v_color, u_alpha) * lighting;
}
