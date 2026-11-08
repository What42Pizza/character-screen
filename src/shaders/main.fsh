#version 450

layout(set = 0, binding = 0) uniform all_uniforms {
	uvec2 target_size;
};
layout(set = 0, binding = 1) uniform texture2D tex;
layout(set = 0, binding = 2) uniform sampler tex_sampler;
layout(set = 1, binding = 0, std430) buffer CharacterMap {
    uint data[];
} character_map;

layout(location = 0) out vec4 frag_color;

void main() {
	frag_color = texture(sampler2D(tex, tex_sampler), gl_FragCoord.xy / target_size);
	//vec3 color = mix(color_1, color_2, texel.r);
	//color = mix(color, color_3, texel.g);
}
