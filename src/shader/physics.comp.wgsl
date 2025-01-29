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
}

struct MouseState {
	position: vec2<f32>,
	clickmask: u32,
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
@group(1) @binding(0) var<storage, read_write> positions: array<vec2<f32>, 16384>;
@group(1) @binding(2) var<storage, read_write> predictions: array<vec2<f32>, 16384>;
@group(1) @binding(1) var<storage, read_write> velocities: array<vec2<f32>, 16384>;
@group(1) @binding(3) var<storage, read_write> densities: array<f32, 16384>;

// rendering
@group(2) @binding(0) var<storage, read> primitives: array<Primitive, 16384>;

