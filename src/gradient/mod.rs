use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct LinearGradient {
    frames: Vec<(f32, Color)>,
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn col_lerp(c0: Color, c1: Color, t: f32) -> Color {
    let c0 = c0.to_srgba();
    let c1 = c1.to_srgba();

    Color::srgba(
        lerp(c0.red, c1.red, t),
        lerp(c0.green, c1.green, t),
        lerp(c0.blue, c1.blue, t),
        lerp(c0.alpha, c1.alpha, t),
    )
}

impl LinearGradient {
    pub fn new(frames: Vec<(f32, Color)>) -> Self {
        Self { frames }
    }

    pub fn sample(&self, t: f32) -> Color {
        assert!((0.0..=1.0).contains(&t), "t must be within 0.0 and 1.0");

        if self.frames.is_empty() {
            panic!("LinearGradient has no frames to sample from");
        }

        if t <= self.frames[0].0 {
            return self.frames[0].1;
        }

        if t >= self.frames[self.frames.len() - 1].0 {
            return self.frames[self.frames.len() - 1].1;
        }

        let mut i = self.frames.len() / 2;

        loop {
            let (t0, c0) = self.frames[i];
            let (t1, c1) = self.frames[i + 1];

            if t0 <= t && t <= t1 {
                let local_t = (t - t0) / (t1 - t0);
                return col_lerp(c0, c1, local_t);
            }

            if t < t0 {
                i /= 2;
            } else {
                i += (self.frames.len() - i) / 2;
            }
        }
    }
}
