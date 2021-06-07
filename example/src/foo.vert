attribute vec3 a_offset;
attribute lowp vec3 a_color;

uniform mat4 u_transform;

varying lowp vec3 v_color;
varying mediump vec3 v_normal;

void main() {
	gl_Position = u_transform * vec4(a_offset, 1.0);
	v_color = a_color;
	v_normal = normalize(a_offset);
}
