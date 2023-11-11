// #import bevy_pbr::mesh_vertex_output MeshVertexOutput
#import bevy_sprite::mesh2d_vertex_output MeshVertexOutput

struct CurveMaterial {
	thickness: f32
}

@group(1) @binding(0)
var<uniform> material: CurveMaterial;

@fragment
fn fragment(mesh: MeshVertexOutput) -> @location(0) vec4<f32> {
	return quadratic_curve(mesh.uv, mesh.color);
}

fn quadratic_curve(point: vec2<f32>, in_color: vec4<f32>) -> vec4<f32> {
	// Gradients
	let px = dpdx(point);
	let py = dpdy(point);
	var color = in_color;

	// Chain Rule
	let fx = (2.0 * point.x * px.x) - px.y;
	let fy = (2.0 * point.x * py.x) - py.y;

	// Signed distance
	let sd = (point.x * point.x - point.y) / sqrt(fx * fx + fy * fy);

	// Linear alpha
	let alpha = material.thickness - abs(sd);
	// let alpha = 0.5 - sd;
	if alpha > 1.0 {
		color.a = 1.0;
	}
	else if alpha < 0.0 {
		discard;
	}
	else {
		color.a = alpha;
	}

	return color;
}