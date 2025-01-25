use crate::prelude::*;
use physics::scene::Scene;

#[cfg(not(feature = "sync"))]
use physics::PhysicsWorkerThread;

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
    pub fn scene(&self) -> &Scene {
        &self.physics
    }

    #[cfg(not(feature = "sync"))]
    pub fn scene(&mut self) -> &Scene {
        self.physics.get()
    }

    #[cfg(feature = "sync")]
    pub fn update(&mut self) {
        if self.reset {
            self.physics.reset();
            self.reset = false;
        }

        self.physics.settings.dtime = self.time.last_instant.elapsed().as_secs_f32();

        if !self.time.paused || self.time.step {
            self.physics.update();
            self.time.step = false;
        }
    }
}
