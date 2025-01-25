use crate::prelude::*;
use physics::scene::Scene;

#[cfg(not(feature = "sync"))]
use physics::PhysicsWorkerThread;

cfg_if! {
    if #[cfg(feature = "sync")] {
        #[derive(Default)]
        pub struct PauseState {
            pub paused: bool,
            pub step: bool,
        }


        pub struct Game {
            pub physics: Scene,

            pub pause: PauseState,
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
                    pause: PauseState::default(),
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
}
