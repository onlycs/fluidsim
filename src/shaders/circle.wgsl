struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
};

const PI: f32 = 3.141592653589793;
const TAU: f32 = 6.283185307179586;
const RADIUS: f32 = 0.25;
const NUM_POINTS: u32 = 33;
const ANGLE_PER_POINT: f32 = TAU / f32(NUM_POINTS);

@vertex
fn vs_main(
    @builtin(vertex_index) index: u32,
) -> VertexOutput {
    var output: VertexOutput;

    if (index % 3 == 0) {
        output.pos = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else {
        let circlei = round(f32(index) / 3.0);
        let angle = circlei * (6.283185307179586 / 33.0);
        let x = cos(angle) * 0.25;
        let y = sin(angle) * 0.25;

        output.pos = vec4<f32>(x, y, 0.0, 1.0);
    }

	return output;
}

@fragment
fn fs_main(vs: VertexOutput) -> @location(0) vec4<f32> {
	return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
