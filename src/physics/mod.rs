pub mod particle;

use async_std::{
    channel::Receiver,
    task::{self, JoinHandle},
};
use ggez::glam::Vec2;
use particle::Particle;
use std::{
    mem, ptr,
    sync::{
        atomic::{AtomicBool, AtomicPtr, Ordering},
        Arc, Mutex,
    },
};

use crate::renderer::message::RendererMessage;

pub const PXSCALE: f32 = 30.0;

#[derive(Clone, Debug)]
pub struct Scene {
    pub particles: Vec<Particle>,
    width: f32,
    height: f32,
}

impl Scene {
    pub fn new(widthpx: f32, heightpx: f32) -> Self {
        let width = widthpx / 2.0 / PXSCALE;
        let height = heightpx / 2.0 / PXSCALE;

        Self {
            particles: (0..20)
                .flat_map(|i| {
                    let i = i as f32 - 10.0;

                    (0..20).map(move |j| {
                        let j = j as f32 - 10.0;

                        Particle::new(
                            Vec2::new(i, j),
                            Vec2::new(0.0, 0.0),
                            Vec2::new(0.0, 0.0),
                            1.0,
                        )
                    })
                })
                .collect(),

            width,
            height,
        }
    }

    pub fn update(&mut self) {
        // todo
    }
}

pub struct PhysicsWorkerThread {
    render: Arc<Mutex<Scene>>,
    saved: Scene,
    thread: task::JoinHandle<()>,
}

impl PhysicsWorkerThread {
    pub fn new(initw: f32, inith: f32, msg: Receiver<RendererMessage>) -> Self {
        let scene = Scene::new(initw, inith);
        let render = Arc::new(Mutex::new(scene.clone()));
        let render_copy = Arc::clone(&render);

        let thread = task::spawn(async move {
            let mut scene = Scene::new(initw, inith);

            loop {
                // receive messages
                while let Ok(msg) = msg.try_recv() {
                    match msg {
                        RendererMessage::Resize(wpx, hpx) => {
                            let w = wpx / 2.0 / PXSCALE;
                            let h = hpx / 2.0 / PXSCALE;

                            scene.width = w;
                            scene.height = h;
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

    pub async fn kill(self) {
        self.thread.cancel().await;
    }
}
