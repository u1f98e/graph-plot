#import bevy_pbr::mesh_vertex_output MeshVertexOutput

struct NodeMaterial {
	color: vec4<f32>,
	radius: f32
}

@group(1) @binding(0)
var<uniform> material: NodeMaterial;

@fragment
fn fragment(mesh: MeshVertexOutput) -> @location(0) vec4<f32> {
	return circle(mesh.position, material.radius, material.color);
}

fn circle(point: vec2<f32>, radius: f32, in_color: vec4<f32>) -> vec4<f32> {
	if()
}