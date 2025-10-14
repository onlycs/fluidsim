use log::LevelFilter;
use simple_logger::SimpleLogger;

#[cfg(debug_assertions)]
const LOG_LEVEL: LevelFilter = LevelFilter::Debug;
#[cfg(not(debug_assertions))]
const LOG_LEVEL: LevelFilter = LevelFilter::Info;

pub fn init() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .with_module_level("fluidsim", LOG_LEVEL)
        .with_module_level("wgpu_hal", LevelFilter::Info)
        .with_module_level("wgpu_core", LevelFilter::Info)
        .with_module_level("eframe", LevelFilter::Warn)
        .with_module_level("egui_wgpu", LevelFilter::Warn)
        .with_module_level("wgpu_hal::gles::egl", LevelFilter::Error)
        .with_module_level("naga::front::spv", LevelFilter::Error)
        .init()
        .unwrap();
}
