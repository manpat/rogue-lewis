#version 130

// uniform mat4 u_proj_view;
attribute vec3 a_vertex;
attribute vec3 a_color;

varying vec3 v_color;

void main() {
	gl_Position = vec4(a_vertex, 1.0);
	v_color = a_color;

	// gl_Position = u_proj_view * vec4(a_vertex, 1.0);
}
