use spirv_std::glam::{vec4, Vec4};

pub struct LinearGradient<const N: usize> {
    frame_positions: [f32; N],
    frame_colors: [Vec4; N],
}

fn lerp(c0: Vec4, c1: Vec4, t: f32) -> Vec4 {
    c0 + (c1 - c0) * t
}

pub fn sample<const N: usize>(g: LinearGradient<N>, t: f32) -> Vec4 {
    if t <= g.frame_positions[0] {
        return g.frame_colors[0] / 255.0;
    }

    if t >= g.frame_positions[N - 1] {
        return g.frame_colors[N - 1] / 255.0;
    }

    let mut i = N / 2;

    loop {
        let t0 = g.frame_positions[i];
        let c0 = g.frame_colors[i];
        let t1 = g.frame_positions[i + 1];
        let c1 = g.frame_colors[i + 1];

        if t0 <= t && t <= t1 {
            let local_t = (t - t0) / (t1 - t0);
            return lerp(c0, c1, local_t) / 255.0;
        }

        if t < t0 {
            i /= 2;
        } else {
            i += (N - i) / 2;
        }
    }
}

pub const VELOCITY: LinearGradient<4> = LinearGradient {
    frame_positions: [0.062, 0.48, 0.65, 1.0],
    frame_colors: [
        vec4(27., 71., 162., 255.),
        vec4(81., 252., 147., 255.),
        vec4(252., 237., 6., 255.),
        vec4(239., 74., 12., 255.),
    ],
};
