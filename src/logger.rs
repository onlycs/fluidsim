use log::LevelFilter;
use skuld::log::SkuldLogger;

pub fn init() {
    SkuldLogger::new("/home/angad/.cache/fluidsim/log.txt".into())
        .unwrap()
        .with_level(LevelFilter::Info)
        .with_module("fluidsim", LevelFilter::Debug)
        .with_module("wgpu_hal", LevelFilter::Error)
        .with_module("wgpu_core", LevelFilter::Warn)
        .with_module("eframe", LevelFilter::Warn)
        .with_module("egui_wgpu", LevelFilter::Warn)
        .with_module("wgpu_hal::gles::egl", LevelFilter::Error)
        .init()
        .unwrap();
}
