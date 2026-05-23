use tracing::Level;
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(debug_assertions)]
const LOG_LEVEL: Level = Level::DEBUG;
#[cfg(not(debug_assertions))]
const LOG_LEVEL: Level = Level::INFO;

pub fn init() {
    #[cfg(debug_assertions)]
    let fmt = tracing_subscriber::fmt::layer().pretty();
    #[cfg(not(debug_assertions))]
    let fmt = tracing_subscriber::fmt::layer().compact();

    let filter = Targets::new()
        .with_default(LOG_LEVEL)
        .with_target("fluidsim", LOG_LEVEL)
        .with_target("wgpu_hal", Level::INFO)
        .with_target("wgpu_core", Level::INFO)
        .with_target("eframe", Level::WARN)
        .with_target("egui_wgpu", Level::WARN)
        .with_target("wgpu_hal::gles::egl", Level::ERROR)
        .with_target("naga::front", Level::WARN);

    tracing_subscriber::registry().with(filter).with(fmt).init();
}
