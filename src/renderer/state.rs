use crate::prelude::*;

#[cfg(not(feature = "sync"))]
use physics::PhysicsWorkerThread;

#[cfg(feature = "sync")]
use physics::scene::Scene;

cfg_if! {
    if #[cfg(feature = "sync")] {
        pub struct TimeState {
            pub paused: bool,
            pub step: bool,
            pub last_instant: Instant,
        }

        pub struct Game {
            pub physics: Scene,

            pub time: TimeState,
            pub reset: bool,
        }
    } else {
        pub struct Game {
            pub physics: PhysicsWorkerThread,
            pub mouse: MouseState,
            pub config: SimSettings,
        }
    }
}

impl Game {
    pub fn new() -> Self {
        cfg_if! {
            if #[cfg(feature = "sync")] {
                Self {
                    physics: Scene::new(),
                    time: TimeState {
                        paused: true,
                        step: false,
                        last_instant: Instant::now()
                    },
                    reset: false,
                }
            } else {
                Self {
                    physics: PhysicsWorkerThread::new(),
                    mouse: MouseState::default(),
                    config: SimSettings::default(),
                }
            }
        }
    }

    #[cfg(feature = "sync")]
    pub fn update(&mut self) {
        if self.reset {
            self.physics.reset();
            self.reset = false;
        }

        let dtime_target = 1. / self.physics.settings.fps;

        if self.time.step {
            self.physics.settings.dtime = dtime_target;
        } else {
            self.physics.settings.dtime = self
                .time
                .last_instant
                .elapsed()
                .as_secs_f32()
                .max(dtime_target)
                .min(1. / 90.);
        }

        self.time.last_instant = Instant::now();

        if !self.time.paused || self.time.step {
            self.physics.update();
            self.time.step = false;
        }
    }
}
