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
}
