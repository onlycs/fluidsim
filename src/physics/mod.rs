pub mod particle;
pub mod prelude;
pub mod scene;

use crate::prelude::*;
use physics::prelude::*;
use std::{
    future::Future,
    sync::{Arc, Mutex},
};

pub const PXSCALE: f32 = 30.0;

pub struct PhysicsWorkerThread {
    render: Arc<Mutex<Scene>>,
    saved: Scene,
    thread: task::JoinHandle<()>,
}

impl PhysicsWorkerThread {
    pub fn new(initw: f32, inith: f32) -> Self {
        let scene = Scene::new(initw, inith);
        let render = Arc::new(Mutex::new(scene.clone()));
        let render_copy = Arc::clone(&render);

        let thread = task::spawn(async move {
            let mut scene = Scene::new(initw, inith);
            let msg = ipc::physics_recv();

            loop {
                // receive messages
                while let Ok(msg) = msg.try_recv() {
                    match msg {
                        ToPhysics::Resize(wpx, hpx) => {
                            let w = wpx / 2.0 / PXSCALE;
                            let h = hpx / 2.0 / PXSCALE;

                            scene.width = w;
                            scene.height = h;
                        }
                        ToPhysics::Gravity(g) => {
                            // todo
                        }
                        ToPhysics::Pause => {
                            // todo
                        }
                        ToPhysics::Step => {
                            // todo
                        }
                    }
                }

                // update scene
                scene.update();

                // store the updated scene
                let mut render_lock = render_copy.lock().unwrap();
                *render_lock = scene.clone();
            }
        });

        Self {
            render,
            thread,
            saved: scene,
        }
    }

    pub fn get(&mut self) -> &Scene {
        if let Ok(render) = self.render.try_lock() {
            self.saved = render.clone();
        }

        &self.saved
    }

    pub async fn kill(self) -> impl Future<Output = Option<()>> {
        self.thread.cancel()
    }
}
