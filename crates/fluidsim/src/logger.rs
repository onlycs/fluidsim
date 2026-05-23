use std::{env, fs, io};

use tracing::Level;
use tracing_subscriber::{
    Layer, filter::Targets, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};

#[cfg(debug_assertions)]
const LOG_LEVEL: Level = Level::DEBUG;
#[cfg(not(debug_assertions))]
const LOG_LEVEL: Level = Level::INFO;

const LEVEL_WGPU: Level = Level::INFO;
const LEVEL_EGUI: Level = Level::WARN;
const LEVEL_HAL: Level = Level::INFO;

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn logfile() -> io::Result<fs::File> {
    let exe = env::current_exe()?;
    let log = exe.parent().unwrap().join("fluidsim.log");

    fs::File::create(&log)
}

#[cfg(target_os = "linux")]
fn logfile() -> io::Result<fs::File> {
    let cd = env::current_dir()?;
    let log = cd.join("fluidsim.log");

    fs::File::create(&log)
}

pub fn init() {
    let filter_env = Targets::new()
        .with_default(LOG_LEVEL)
        .with_target("fluidsim", LOG_LEVEL)
        .with_target("wgpu_hal", LEVEL_WGPU)
        .with_target("wgpu_core", LEVEL_WGPU)
        .with_target("eframe", LEVEL_EGUI)
        .with_target("egui_wgpu", LEVEL_EGUI)
        .with_target("wgpu_hal::gles::egl", LEVEL_HAL)
        .with_target("naga::front", LEVEL_HAL);

    let filter_dbg = Targets::new()
        .with_default(Level::DEBUG)
        .with_target("fluidsim", Level::DEBUG)
        .with_target("wgpu_hal", Level::DEBUG)
        .with_target("wgpu_core", Level::DEBUG)
        .with_target("eframe", Level::DEBUG)
        .with_target("egui_wgpu", Level::DEBUG)
        .with_target("wgpu_hal::gles::egl", Level::DEBUG)
        .with_target("naga::front", Level::DEBUG);

    let file = fmt::layer()
        .with_writer(logfile().unwrap())
        .with_ansi(false)
        .with_filter(filter_dbg);

    #[cfg(debug_assertions)]
    let fmt = fmt::layer().pretty().with_filter(filter_env);
    #[cfg(not(debug_assertions))]
    let fmt = fmt::layer().compact().with_filter(filter_env);

    tracing_subscriber::registry().with(fmt).with(file).init();
}
