use crate::prelude::*;
use physics::scene::Scene;

#[cfg(not(feature = "sync"))]
use physics::PhysicsWorkerThread;

cfg_if! {
    if #[cfg(feature = "sync")] {
        pub struct Game {
            pub physics: Scene,
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
                    physics: Scene::new()
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
