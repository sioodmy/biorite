#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings
#import bevy_pbr::mesh_functions

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) index: i32,
    @location(4) light_level: f32,
    @location(5) ambient_occlusion: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
    @location(3) index: i32,
    @location(4) light_level: f32,
    @location(5) ambient_occlusion: f32,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    var model = mesh.model;
    out.world_normal = mesh_normal_local_to_world(vertex.normal);
    out.world_position = mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));
    out.uv = vertex.uv;

    out.clip_position = mesh_position_world_to_clip(out.world_position);
    out.index = vertex.index;
    out.light_level = vertex.light_level;
    out.ambient_occlusion = vertex.ambient_occlusion;
    return out;
}