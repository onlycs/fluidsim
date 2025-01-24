use crate::prelude::*;
use shared::MouseState;

cfg_if! {
    if #[cfg(feature = "sync")] {
        use physics::scene::Scene;
    } else {
        use physics::PhysicsWorkerThread;
    }
}

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
}
