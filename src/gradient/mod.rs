use wgpu::Color;

#[derive(Clone, Debug)]
pub struct LinearGradient {
    frames: Vec<(f32, Color)>,
}

fn lerp(c0: Color, c1: Color, t: f64) -> Color {
    Color {
        r: c0.r + (c1.r - c0.r) * t,
        g: c0.g + (c1.g - c0.g) * t,
        b: c0.b + (c1.b - c0.b) * t,
        a: c0.a + (c1.a - c0.a) * t,
    }
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
                return lerp(c0, c1, local_t as f64);
            }

            if t < t0 {
                i /= 2;
            } else {
                i += (self.frames.len() - i) / 2;
            }
        }
    }
}
