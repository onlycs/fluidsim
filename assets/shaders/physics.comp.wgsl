struct Settings {
	dtime: f32,
	
	gravity: f32,
	collision_damping: f32,

	smoothing_radius: f32,
	target_density: f32,
	pressure_multiplier: f32,
	mass: f32,

	interaction_radius: f32,
	interaction_strength: f32,

	viscosity_strength: f32,

	num_particles: u32,
	particle_radius: f32,

	window_size: vec2<f32>,
	_pad: u32,
}

struct MouseState {
	position: vec2<f32>,
	clickmask: u32,
	_pad: u32
}

struct Primitive {
    color: vec4<f32>,
    translate: vec2<f32>,
    z_index: i32,
	_pad: u32,
}

// user configurable/mutable stuff
@group(0) @binding(0) var<uniform> settings: Settings;
@group(0) @binding(1) var<uniform> mouse: MouseState;

// simulation
@group(1) @binding(0) var<storage, read_write> positions: array<vec2<f32>>;
@group(1) @binding(2) var<storage, read_write> predictions: array<vec2<f32>>;
@group(1) @binding(1) var<storage, read_write> velocities: array<vec2<f32>>;
@group(1) @binding(3) var<storage, read_write> densities: array<f32>;

// rendering
@group(2) @binding(0) var<storage, read_write> primitives: array<Primitive>;

// spatial hash
@group(3) @binding(0) var<storage, read_write> lookup: array<u32>;
@group(3) @binding(1) var<storage, read_write> starts: array<u32>;

// constants
const SCALE: f32 = 100.0;

// mouse utils
fn mouse_left() -> bool {
	return (mouse.clickmask & 1) != 0;
}

fn mouse_right() -> bool {
	return (mouse.clickmask & 2) != 0;
}

fn mouse_intensity() -> f32 {
	if !mouse_left() && !mouse_right() {
		return 0.0;
	}

	if mouse_left() {
		return 1.0;
	} else {
		return -1.0;
	}
}

@compute
@workgroup_size(64)
fn external_forces(
	@builtin(global_invocation_id) id: vec3<u32>
) {
	let index = id.x;
	if (index >= settings.num_particles) {
		return;
	}

	let gravity = vec2<f32>(0.0, settings.gravity);
	var ext_accel: vec2<f32>;

	if mouse_left() || mouse_right() {
		let mpos = mouse.position / SCALE - settings.window_size / SCALE / 2.0;
		let offset = mpos - positions[index];
		let dist2 = dot(offset, offset);

		if dist2 < settings.interaction_radius * settings.interaction_radius {
			let dist = sqrt(dist2);
			let edge = dist / settings.interaction_radius;
			let center = 1.0 - edge;
			let dir = offset / dist;
			let strength = settings.interaction_strength * mouse_intensity();

			// reduce gravity when interacting with mouse.
			let gweight = 1.0 - saturate(center * (strength / 10.0));
			let accel = gravity * gweight + dir * strength * center;

			ext_accel = accel - velocities[index] * center;
		}
	} else {
		ext_accel = gravity;
	}

	velocities[index] += ext_accel * settings.dtime;
}

@compute
@workgroup_size(64)
fn update_predictions(
	@builtin(global_invocation_id) id: vec3<u32>
) {
	let index = id.x;
	if (index >= settings.num_particles) {
		return;
	}

	predictions[index] = positions[index] + velocities[index] * (1.0 / 120.0);
}

@compute
@workgroup_size(64)
fn update_positions(
	@builtin(global_invocation_id) id: vec3<u32>
) {
	let index = id.x;
	if (index >= settings.num_particles) {
		return;
	}

	positions[index] += velocities[index] * settings.dtime;
}

@compute
@workgroup_size(64)
fn copy_to_prims(
	@builtin(global_invocation_id) id: vec3<u32>
) {
	let index = id.x;
	if (index >= settings.num_particles) {
		return;
	}

	primitives[index].translate = positions[index] * SCALE;
	primitives[index].color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
}