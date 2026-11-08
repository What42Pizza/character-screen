#version 450

#define CHAR_WIDTH 10
#define CHAR_HEIGHT 12
#define CHAR_SIZE uvec2(CHAR_WIDTH, CHAR_HEIGHT)

layout(set = 0, binding = 0) uniform all_uniforms {
	uvec2 charmap_size;
	uvec2 target_size;
};
layout(set = 0, binding = 1) uniform texture2D tex;
layout(set = 0, binding = 2) uniform sampler tex_sampler;
#define char_atlas sampler2D(tex, tex_sampler)
layout(set = 1, binding = 0, std430) readonly buffer CharacterMap {
    uint data[];
} charmap;

layout(location = 0) out vec4 frag_color;

void main() {
	uvec2 texel_coord = uvec2(gl_FragCoord.xy);
	
	uvec2 charmap_pos = texel_coord / CHAR_SIZE;
	if (charmap_pos.x >= charmap_size.x || charmap_pos.y >= charmap_size.y) {
		discard;
	}
	
	uint char_data = charmap.data[charmap_pos.x + charmap_pos.y * charmap_size.x];
	uint char_index = char_data & 0x000000FF;
	
	uvec2 in_char_coord = texel_coord % CHAR_SIZE;
	vec2 char_tex_coord = uvec2((char_index & 0x0F) * CHAR_WIDTH, (char_index >> 4) * CHAR_HEIGHT) + in_char_coord + 0.5;
	vec4 char_tex_color = texture(char_atlas, char_tex_coord / (CHAR_SIZE * 16));
	frag_color = char_tex_color;
	
}
