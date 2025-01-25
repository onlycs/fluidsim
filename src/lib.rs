#![feature(
    error_generic_member_access,
    never_type,
    let_chains,
    if_let_guard,
    trait_alias,
    trivial_bounds,
    generic_arg_infer
)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate cfg_if;

extern crate async_std;
extern crate egui;
extern crate egui_wgpu;
extern crate glam;
extern crate itertools;
extern crate lazy_static;
extern crate lyon;
extern crate rand;
extern crate rayon;
extern crate skuld;
extern crate thiserror;
extern crate wgpu;
extern crate winit;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        extern crate console_error_panic_hook;
        extern crate console_log;
        extern crate wasm_bindgen;
        extern crate wasm_bindgen_futures;
        extern crate web_sys;

        use wasm_bindgen::prelude::*;
    }
}

mod error;
mod gradient;
mod ipc;
mod logger;
mod physics;
mod prelude;
mod renderer;

use error::InitError;
use renderer::SimRenderer;
use winit::event_loop::EventLoop;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    main().unwrap()
}

#[cfg(all(target_arch = "wasm32", not(feature = "sync")))]
compile_error!("`sync` feature must be turned on for wasm32 support");

pub fn main() -> Result<(), InitError> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Info).unwrap();
        } else {
            logger::init();
        }
    }

    info!("Starting up");

    let event_loop = EventLoop::builder().build()?;
    let app = Box::leak(Box::new(SimRenderer::new()));

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    event_loop.run_app(app)?;

    Ok(())
}
