use crate::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TimeState {
    Paused,
    /// Running(time of last-ran frame)
    Running(Instant),
    StepRequested,
}

impl TimeState {
    pub(crate) fn toggle(&mut self) {
        *self = match self {
            TimeState::Paused | TimeState::StepRequested => TimeState::Running(Instant::now()),
            TimeState::Running(_) => TimeState::Paused,
        };
    }

    pub(crate) fn pause(&mut self) {
        *self = TimeState::Paused;
    }

    pub(crate) fn step(&mut self) {
        if let TimeState::Paused = self {
            *self = TimeState::StepRequested;
        } else {
            warn!("Cannot step while not paused");
        }
    }
}

pub(crate) struct SimulationState {
    pub(crate) time: TimeState,
    pub(crate) gfx: GraphicsSettings,
    pub(crate) init: InitialConditions,
}

impl SimulationState {
    pub(crate) fn new() -> Self {
        Self {
            gfx: GraphicsSettings::default(),
            init: InitialConditions::default(),
            time: TimeState::Paused,
        }
    }

    /// Calculates the change in time for this simulation frame.
    pub(crate) fn dtime(&mut self) -> f32 {
        match &mut self.time {
            TimeState::Running(prv) => {
                let now = Instant::now();
                let dtime = now.duration_since(*prv).as_secs_f64();
                *prv = now;
                (dtime as f32).min(1.0 / 60.0) * self.gfx.speed
            }
            TimeState::Paused => 0.0,
            TimeState::StepRequested => {
                self.time = TimeState::Paused;
                self.gfx.step_time / 1000.0
            }
        }
    }
}
