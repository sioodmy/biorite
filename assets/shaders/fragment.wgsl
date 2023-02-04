@group(1) @binding(0)
var my_array_texture: texture_2d_array<f32>;
@group(1) @binding(1)
var my_array_texture_sampler: sampler;

@fragment
fn fragment(
@location(0) world_position: vec4<f32>,
@location(1) world_normal: vec3<f32>,
@location(2) uv: vec2<f32>,
@location(3) index: i32,
@location(4) light_level: f32,
@location(5) ambient_occlusion: f32,
) -> @location(0) vec4<f32> {
    var color = textureSample(my_array_texture, my_array_texture_sampler, uv, index);
    if (color.a < 0.5) {
        discard;
    }
    color *= light_level * ((ambient_occlusion + 0.5) / 2.0);

    color.a = 1.0;
    return color;
}