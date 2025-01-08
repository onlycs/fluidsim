pub mod prelude;
pub mod scene;
pub mod settings;

use crate::prelude::*;
use async_std::sync::{Arc, Mutex};
use physics::prelude::*;
use std::{
    future::Future,
    time::{Duration, Instant},
};

pub struct PhysicsWorkerThread {
    render: Arc<Mutex<Scene>>,
    saved: Scene,
    thread: task::JoinHandle<()>,
}

impl PhysicsWorkerThread {
    pub fn new() -> Self {
        let scene = Scene::new();
        let render = Arc::new(Mutex::new(scene.clone()));
        let render_copy = Arc::clone(&render);

        let thread = task::spawn(async move {
            let mut scene = Scene::new();
            let msg = ipc::physics_recv();

            let render = Arc::clone(&render_copy);

            let mut pause = true;
            let mut pause_next = false;
            let mut timer = Instant::now();

            loop {
                // sleep
                let el = timer.elapsed();
                let durpt = Duration::from_secs_f32(scene.settings.tick_delay.get::<second>());
                let sleep = durpt.saturating_sub(el);

                if !sleep.is_zero() {
                    task::sleep(sleep).await;
                } else {
                    warn!("Physics thread is running behind: {:?}", el - durpt);
                }

                timer = Instant::now();

                // receive messages
                while let Ok(msg) = msg.try_recv() {
                    match msg {
                        ToPhysics::Settings(s) => {
                            scene.update_settings(s);
                        }
                        ToPhysics::Pause => {
                            pause = !pause;
                            pause_next = false;
                        }
                        ToPhysics::Step if pause => {
                            pause = false;
                            pause_next = true;
                        }
                        ToPhysics::Step => {
                            warn!("Received step message while not paused");
                        }
                        ToPhysics::Reset => {
                            scene.reset();
                        }
                    }
                }

                // update scene
                if pause_next {
                    scene.update();

                    pause = true;
                    pause_next = false;
                } else if !pause {
                    scene.update();
                }

                // store the updated scene
                let mut render_lock = render.lock().await;
                *render_lock = scene.clone();
                drop(render_lock);
            }
        });

        Self {
            render,
            thread,
            saved: scene,
        }
    }

    pub fn get(&mut self) -> &Scene {
        if let Some(render) = self.render.try_lock() {
            self.saved = render.clone();
        }

        &self.saved
    }

    pub async fn kill(self) -> impl Future<Output = Option<()>> {
        self.thread.cancel()
    }
}
