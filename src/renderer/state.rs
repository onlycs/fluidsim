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

pub struct Game {
    pub physics: Scene,

    pub time: TimeState,
    pub reset: bool,
}

impl Game {
    pub fn new() -> Self {
        Self {
            physics: Scene::new(),
            time: TimeState {
                paused: true,
                step: false,
                last_instant: Instant::now(),
            },
            reset: false,
        }
    }

    pub fn update(&mut self) {
        if self.reset {
            self.physics.reset();
            self.reset = false;
            self.time.paused = true;
        }

        if self.time.step {
            self.physics.settings.dtime = self.physics.settings.step_time / 1e3;
        } else {
            let el = self.time.last_instant.elapsed().as_secs_f32();
            let speed = self.physics.settings.speed;
            let sspf = self.physics.settings.steps_per_frame;
            let maxed = (el * speed / sspf as f32).min(1. / 90.);

            self.physics.settings.dtime = maxed;
        }

        self.time.last_instant = Instant::now();

        if !self.time.paused || self.time.step {
            for _ in 0..self.physics.settings.steps_per_frame {
                self.physics.update();
            }

            self.time.step = false;
        }
    }
}
