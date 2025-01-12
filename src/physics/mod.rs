pub mod scene;
pub mod settings;

use crate::prelude::*;
use async_std::sync::{Arc, Mutex};
use scene::Scene;

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

#[cfg(target_arch = "wasm32")]
use web_time::Instant;

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
        let fut = async move {
            let mut scene = Scene::new();
            let msg = ipc::physics_recv();

            let render = Arc::clone(&render_copy);

            let mut pause = true;
            let mut timer = Instant::now();
            let mut spt_target = 1. / scene.settings.fps;

            'physics: loop {
                // receive messages
                while let Ok(msg) = msg.try_recv() {
                    match msg {
                        ToPhysics::Settings(s) => {
                            spt_target = 1. / s.fps;
                            scene.update_settings(s);
                        }
                        ToPhysics::Pause => {
                            info!("Toggling pause");
                            pause = !pause;
                        }
                        ToPhysics::Step if pause => {
                            scene.settings.dtime = spt_target;
                            scene.update();
                        }
                        ToPhysics::Step => {
                            warn!("Received step message while not paused");
                        }
                        ToPhysics::Reset => {
                            info!("Resetting scene");
                            scene.reset();
                            continue 'physics;
                        }
                        ToPhysics::UpdateMouse(mouse) => {
                            scene.mouse = mouse;
                        }
                        ToPhysics::Kill => {
                            info!("Physics thread killed");
                            break 'physics;
                        }
                    }
                }

                // sleep
                let el = timer.elapsed();
                timer = Instant::now();

                if el.as_secs_f32() < spt_target {
                    #[cfg(not(target_arch = "wasm32"))]
                    task::sleep(Duration::from_secs_f32(spt_target) - el).await;

                    #[cfg(target_arch = "wasm32")]
                    gloo_timers::future::TimeoutFuture::new(
                        ((spt_target - el.as_secs_f32()) * 1000.) as u32,
                    )
                    .await;
                }

                scene.settings.dtime = spt_target;

                // update scene
                if !pause {
                    scene.update();
                }

                // store the updated scene
                let mut render_lock = render.lock().await;
                *render_lock = scene.clone();
                drop(render_lock);

                gloo_timers::future::TimeoutFuture::new(1).await;
            }
        };

        #[cfg(not(target_arch = "wasm32"))]
        let thread = task::spawn(fut);

        #[cfg(target_arch = "wasm32")]
        let thread = task::spawn_local(fut);

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
}
