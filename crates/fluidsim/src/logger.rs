use log::LevelFilter;
use skuld::log::SkuldLogger;

#[cfg(debug_assertions)]
const LOG_LEVEL: LevelFilter = LevelFilter::Debug;
#[cfg(not(debug_assertions))]
const LOG_LEVEL: LevelFilter = LevelFilter::Info;

pub fn init() {
    SkuldLogger::new("/home/angad/.cache/fluidsim/log.txt".into())
        .unwrap()
        .with_level(LevelFilter::Info)
        .with_module("fluidsim", LOG_LEVEL)
        .with_module("wgpu_hal", LevelFilter::Error)
        .with_module("wgpu_core", LevelFilter::Warn)
        .with_module("eframe", LevelFilter::Warn)
        .with_module("egui_wgpu", LevelFilter::Warn)
        .with_module("wgpu_hal::gles::egl", LevelFilter::Error)
        .init()
        .unwrap();
}
