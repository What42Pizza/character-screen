#version 450

// these are given by the rust code to naga:
//#define CHAR_WIDTH 10
//#define CHAR_HEIGHT 12
#define CHAR_SIZE uvec2(CHAR_WIDTH, CHAR_HEIGHT)

layout(set = 0, binding = 0) uniform all_uniforms {
	uvec2 charmap_size;
};
layout(set = 0, binding = 1) uniform texture2D char_atlas_tex;
layout(set = 0, binding = 2) uniform texture2D color_lut_tex;
layout(set = 0, binding = 3) uniform sampler nearest_sampler;
#define char_atlas sampler2D(char_atlas_tex, nearest_sampler)
#define color_lut sampler2D(color_lut_tex, nearest_sampler)
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
	
	vec2 color_1_coord = uvec2((char_data & 0x00000F00) >> 8 , (char_data & 0x0000F000) >> 12) + 0.5;
	vec3 color_1 = texture(color_lut, color_1_coord / 16.0).rgb;
	vec2 color_2_coord = uvec2((char_data & 0x000F0000) >> 16, (char_data & 0x00F00000) >> 20) + 0.5;
	vec3 color_2 = texture(color_lut, color_2_coord / 16.0).rgb;
	vec2 color_3_coord = uvec2((char_data & 0x0F000000) >> 24, (char_data & 0xF0000000) >> 28) + 0.5;
	vec3 color_3 = texture(color_lut, color_3_coord / 16.0).rgb;
	
	frag_color.rgb = mix(color_1, color_2, char_tex_color.r);
	frag_color.rgb = mix(frag_color.rgb, color_3, char_tex_color.g);
	frag_color.a = char_tex_color.a;
	
}
