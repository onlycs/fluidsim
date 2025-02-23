use crate::prelude::*;

pub struct TimeState {
    paused: bool,
    step: bool,
    last_instant: Instant,
}

impl TimeState {
    pub fn play_pause(&mut self) {
        self.paused = !self.paused;
        self.last_instant = Instant::now();
    }

    pub fn step(&mut self) {
        if self.paused {
            self.step = true;
        } else {
            warn!("Cannot step while not paused");
        }
    }
}

pub struct GameState {
    pub time: TimeState,
    pub gfx: GraphicsSettings,
    pub init: InitialConditions,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            gfx: GraphicsSettings::default(),
            init: InitialConditions::default(),
            time: TimeState {
                paused: true,
                step: false,
                last_instant: Instant::now(),
            },
        }
    }

    pub fn can_update(&mut self) -> bool {
        if self.time.paused {
            if self.time.step {
                self.time.step = false;
                true
            } else {
                false
            }
        } else {
            true
        }
    }

    pub fn dtime(&mut self) -> f32 {
        if self.time.paused {
            return self.gfx.step_time / 1000.0;
        }

        let now = Instant::now();
        let dtime = now.duration_since(self.time.last_instant).as_secs_f64();
        self.time.last_instant = now;

        dtime as f32 * self.gfx.speed
    }
}
