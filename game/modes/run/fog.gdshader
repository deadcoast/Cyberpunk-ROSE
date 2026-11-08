shader_type canvas_item;
render_mode blend_mix;

// The fog of war and light as a 1-bit ordered dither in Void over the scene, at the art's own pixel grid
// (the viewport is 640x360 integer-scaled, so FRAGCOORD is an art pixel). One data texel per tile:
//   R  light 0-15 (*17)      G  fog: 0 unseen, 128 remembered, 255 visible      B  noise 0-15 (*17)
// Unseen tiles are solid Void; remembered tiles are a heavy dither; visible tiles dither by how dark they are,
// never below a floor so what you can see stays readable. Noise paints a sparse cyan dither that fades with the
// field: the Mark of the Ninja ring, honest about pipes.

uniform sampler2D data : filter_nearest, repeat_disable;
uniform vec4 void_col : source_color = vec4(0.0275, 0.0275, 0.047, 1.0);
uniform vec4 noise_col : source_color = vec4(0.098, 0.902, 1.0, 1.0);
uniform float visible_floor : hint_range(0.0, 1.0) = 0.55;
uniform float remembered_dark : hint_range(0.0, 1.0) = 0.8;

const float BAYER[16] = float[16](0.0, 8.0, 2.0, 10.0, 12.0, 4.0, 14.0, 6.0, 3.0, 11.0, 1.0, 9.0, 15.0, 7.0, 13.0, 5.0);

void fragment() {
	vec3 d = texture(data, UV).rgb;
	float light = d.r;
	float fog = d.g;
	float noise = d.b;
	int bx = int(mod(FRAGCOORD.x, 4.0));
	int by = int(mod(FRAGCOORD.y, 4.0));
	float t = (BAYER[bx + by * 4] + 0.5) / 16.0;
	float darkness;
	if (fog < 0.25) {
		darkness = 1.0;
	} else if (fog < 0.75) {
		darkness = remembered_dark;
	} else {
		darkness = visible_floor * (1.0 - light);
	}
	vec4 out_col = vec4(0.0);
	if (darkness >= 1.0 || darkness > t) {
		out_col = void_col;
	}
	// noise flash on tiles the sound reached, drawn only where the player can see
	if (noise > 0.0 && fog >= 0.75 && t > 1.0 - noise * 0.3) {
		out_col = noise_col;
	}
	COLOR = out_col;
}
