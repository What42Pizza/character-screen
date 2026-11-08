#version 450

void main() {
	uint id = gl_VertexIndex;
	gl_Position = vec4(1.0);
	if (id == 0u) gl_Position = vec4(-1.0, -1.0, 0.5, 1.0);
	if (id == 1u) gl_Position = vec4(-1.0,  3.0, 0.5, 1.0);
	if (id == 2u) gl_Position = vec4( 3.0, -1.0, 0.5, 1.0);
}
