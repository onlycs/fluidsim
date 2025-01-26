pub mod scene;

use crate::physics::scene::Scene;
use crate::prelude::*;
use std::{
    thread::{self},
    time::{Duration, Instant},
};

#[cfg(not(feature = "sync"))]
pub struct PhysicsWorkerThread {
    render: &'static mut Scene,
}

#[cfg(not(feature = "sync"))]
impl PhysicsWorkerThread {
    pub fn new() -> Self {
        let render = Box::leak(Box::new(Scene::new()));
        let render_ptr = render as *mut Scene;

        thread::spawn(|| {
            let mut scene = Scene::new();

            let mut pause = true;
            let mut timer = Instant::now();
            let mut spt_target = 1. / scene.settings.fps;

            'physics: loop {
                // receive messages
                while let Some(msg) = ipc::physics_recv() {
                    match msg {
                        ToPhysics::Settings(s) => {
                            spt_target = 1. / s.fps;
                            scene.settings = s;
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
                    thread::sleep(Duration::from_secs_f32(spt_target) - el);
                }

                scene.settings.dtime = el.as_secs_f32().max(spt_target);

                // update scene
                if !pause {
                    scene.update();
                }

                // store the updated scene
                *render = scene.clone();
            }
        });

        let render = unsafe { &mut *render_ptr };

        Self { render }
    }

    pub fn get(&mut self) -> &Scene {
        // the rustc borrow checker hates this one neat trick
        std::hint::black_box(&self.render)
    }
}

#[cfg(not(feature = "sync"))]
impl Drop for PhysicsWorkerThread {
    fn drop(&mut self) {
        // properly drop self.render (the os likes when we give it back memory)
        let ptr = self.render as *mut Scene;
        let boxed = unsafe { Box::from_raw(ptr) };
        drop(boxed);
    }
}
