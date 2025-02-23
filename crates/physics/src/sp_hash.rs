use gpu_shared::ARRAY_LEN;
use spirv_std::glam::{ivec2, vec2, IVec2, Vec2};
use spirv_std::num_traits::real::Real;

pub const NEIGHBORS: [IVec2; 9] = [
    ivec2(-1, -1),
    ivec2(-1, 0),
    ivec2(-1, 1),
    ivec2(0, -1),
    ivec2(0, 0),
    ivec2(0, 1),
    ivec2(1, -1),
    ivec2(1, 0),
    ivec2(1, 1),
];

pub fn pos_to_cell(pos: Vec2, cell_size: f32) -> IVec2 {
    let x = (pos.x / cell_size).floor();
    let y = (pos.y / cell_size).floor();

    ivec2(x as i32, y as i32)
}

pub fn cell_key(IVec2 { x, y }: IVec2, num_particles: u32) -> u32 {
    let px = 17;
    let py = 31;
    let h = (x * px + y * py).rem_euclid(num_particles as i32);

    h as u32
}

pub fn pos_to_key(pos: Vec2, cell_size: f32, num_particles: u32) -> u32 {
    cell_key(pos_to_cell(pos, cell_size), num_particles)
}

pub fn get_by_key(key: u32, starts: &[u32; ARRAY_LEN], num_particles: u32) -> (u32, u32) {
    if key >= num_particles {
        return (0, 0);
    }

    let idx = starts[key as usize];
    if idx == u32::MAX {
        return (0, 0);
    }

    let mut end = num_particles;

    for i in key + 1..end {
        if starts[i as usize] != u32::MAX {
            end = starts[i as usize];
            break;
        }
    }

    if end <= idx {
        return (0, 0);
    }

    (idx, end)
}
