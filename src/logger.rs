use log::LevelFilter;
use skuld::log::SkuldLogger;

pub fn init() {
    SkuldLogger::new("/home/angad/.cache/fluidsim/log.txt".into())
        .unwrap()
        .with_level(LevelFilter::Info)
        .with_module("fluidsim", LevelFilter::Debug)
        .with_module("wgpu_hal", LevelFilter::Warn)
        .with_module("wgpu_core", LevelFilter::Warn)
        .init()
        .unwrap();
}
