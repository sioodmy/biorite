#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings

@group(1) @binding(0)
var my_array_texture: texture_2d_array<f32>;
@group(1) @binding(1)
var my_array_texture_sampler: sampler;

#import bevy_pbr::mesh_functions
struct FragmentInput {
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) index: i32,
    @location(4) light: f32,
};

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
#ifdef VERTEX_UVS
    @location(2) uv: vec2<f32>,
#endif
    @location(3) index: i32,
    @location(4) light: f32,
    @location(5) ao: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
    
    @location(3) index: i32,
    @location(4) light: f32,
    @location(5) ao: f32,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.uv = vertex.uv;
    var model = mesh.model;
    out.world_position = mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));
    out.world_normal = mesh_normal_local_to_world(vertex.normal);
    out.clip_position = mesh_position_world_to_clip(out.world_position);
    out.index = vertex.index;
    out.light = vertex.light;
    out.ao = vertex.ao;
    return out;
}

@fragment
fn fragment(in: FragmentInput

) -> @location(0) vec4<f32> {
    var color = textureSample(my_array_texture, my_array_texture_sampler, in.uv, in.index);
    // return color * in.light * ((in.ao + 0.5) / 3.5);
    return color * in.light * ((in.ao + 0.5) / 3.5);
}