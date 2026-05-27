use std::sync::Mutex;

use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, SwashCache, TextArea,
    TextAtlas, TextBounds, TextRenderer, Viewport, Weight, cosmic_text::Align,
};
use wgpu::{CommandEncoder, MultisampleState, TextureView};

use super::{graphics::GraphicsContext, shader::pipelines::ComputeShaderPerformance};
use crate::{prelude::*, renderer::state::SimulationState};

const FONT_SIZE: f32 = 18.0;
const LINE_HEIGHT: f32 = 24.0;

const JETBRAINS_MONO: &[u8] = include_bytes!(concat!(
    env!("CARGO_WORKSPACE_DIR"),
    "assets/font/JetBrainsMono-Light.ttf"
));

#[derive(Debug, Snafu)]
pub(crate) enum TextError {
    #[snafu(display("At {location}: glyphon: prepare error\n{source}"))]
    Prepare {
        source: glyphon::PrepareError,
        #[snafu(implicit)]
        location: Location,
    },
    #[snafu(display("At {location}: glyphon: render error\n{source}"))]
    Render {
        source: glyphon::RenderError,
        #[snafu(implicit)]
        location: Location,
    },
}

pub(crate) struct PerformanceDisplay {
    font_system: FontSystem,
    swash_cache: SwashCache,
    viewport: Viewport,
    atlas: TextAtlas,
    renderer: TextRenderer,
    buffer_fps: Buffer,
    buffer_xyzrpy: Buffer,

    pub(crate) data: Arc<Mutex<ComputeShaderPerformance>>,
    last_data: ComputeShaderPerformance,
    pub(crate) show: bool,

    timer: Instant,
    frames: usize,
    fps: f32,
    scale: f32,
}

impl PerformanceDisplay {
    pub(crate) fn new(wgpu: &GraphicsContext) -> Self {
        let size = wgpu.window.inner_size().to_uvec2();
        let scale = wgpu.window.scale_factor() as f32;

        let mut font_system = FontSystem::new();
        font_system.db_mut().load_font_data(JETBRAINS_MONO.to_vec());

        let swash_cache = SwashCache::new();
        let cache = Cache::new(&wgpu.device);
        let viewport = Viewport::new(&wgpu.device, &cache);
        let mut atlas = TextAtlas::new(&wgpu.device, &wgpu.queue, &cache, wgpu.config.format);
        let renderer =
            TextRenderer::new(&mut atlas, &wgpu.device, MultisampleState::default(), None);

        let buffer_fps = Self::make_buffer(&mut font_system, size, scale);
        let buffer_xyzrpy = Self::make_buffer(&mut font_system, size, scale);

        Self {
            font_system,
            swash_cache,
            viewport,
            atlas,
            renderer,
            buffer_fps,
            buffer_xyzrpy,
            show: false,
            data: Arc::new(Mutex::new(ComputeShaderPerformance::default())),
            last_data: ComputeShaderPerformance::default(),
            timer: Instant::now(),
            frames: 0,
            fps: 0.0,
            scale,
        }
    }

    fn make_buffer(font_system: &mut FontSystem, size: UVec2, scale: f32) -> Buffer {
        let mut buf = Buffer::new(font_system, Metrics::new(FONT_SIZE, LINE_HEIGHT));
        buf.set_size(
            font_system,
            Some(size.x as f32 * scale),
            Some(size.y as f32 * scale),
        );
        buf
    }

    fn attrs() -> Attrs<'static> {
        Attrs::new()
            .family(Family::Name("JetBrains Mono"))
            .weight(Weight::LIGHT)
    }

    pub(crate) fn resize(&mut self, size: UVec2, scale: f32) {
        let (w, h) = (Some(size.x as f32 * scale), Some(size.y as f32 * scale));
        self.buffer_fps.set_size(&mut self.font_system, w, h);
        self.buffer_xyzrpy.set_size(&mut self.font_system, w, h);
        self.scale = scale;
    }

    pub(crate) fn toggle(&mut self) {
        self.show = !self.show;
    }

    pub(crate) fn update(&mut self) {
        self.frames += 1;
        if self.timer.elapsed().as_secs_f32() > 1.0 {
            self.fps = self.frames as f32 / self.timer.elapsed().as_secs_f32();
            self.frames = 0;
            self.last_data = *self.data.lock().unwrap();
            self.timer = Instant::now();
        }
    }

    #[allow(clippy::too_many_lines)]
    pub(crate) fn render(
        &mut self,
        wgpu: &GraphicsContext,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        state: &SimulationState,
    ) -> Result<(), TextError> {
        let fps_text = if self.show {
            format!("{}\nFPS: {:.2}", self.last_data, self.fps)
        } else {
            format!("FPS: {:.2}", self.fps)
        };

        self.buffer_fps.set_text(
            &mut self.font_system,
            &fps_text,
            &Self::attrs(),
            glyphon::Shaping::Advanced,
            Some(Align::Left),
        );
        self.buffer_fps
            .shape_until_scroll(&mut self.font_system, false);

        let (r, p, y) = state.player.q.to_euler(glam::EulerRot::XYZ);
        let xyzrpy_text = format!(
            "position: ({:.2}, {:.2}, {:.2})\nrotation: ({:.2}, {:.2}, {:.2})\nquat: ({:.2}, {:.2}, {:.2}, {:.2})",
            state.player.translate.x,
            state.player.translate.y,
            state.player.translate.z,
            r.to_degrees(),
            p.to_degrees(),
            y.to_degrees(),
            state.player.q.x,
            state.player.q.y,
            state.player.q.z,
            state.player.q.w,
        );

        self.buffer_xyzrpy.set_text(
            &mut self.font_system,
            &xyzrpy_text,
            &Self::attrs(),
            glyphon::Shaping::Advanced,
            Some(Align::Right),
        );
        self.buffer_xyzrpy
            .shape_until_scroll(&mut self.font_system, false);

        let UVec2 { x: w, y: h } = wgpu.window.inner_size().to_uvec2();
        let fps_lines = fps_text.lines().count() as f32;
        let xyz_lines = xyzrpy_text.lines().count() as f32;

        self.viewport.update(
            &wgpu.queue,
            Resolution {
                width: w,
                height: h,
            },
        );

        let Self {
            font_system,
            swash_cache,
            atlas,
            renderer,
            buffer_fps,
            buffer_xyzrpy,
            viewport,
            ..
        } = self;

        renderer
            .prepare(
                &wgpu.device,
                &wgpu.queue,
                font_system,
                atlas,
                viewport,
                [
                    TextArea {
                        buffer: buffer_fps,
                        left: 10.0,
                        top: h as f32 - 10.0 - LINE_HEIGHT * fps_lines,
                        scale: 1.0,
                        bounds: TextBounds {
                            left: 0,
                            top: 0,
                            right: w as i32,
                            bottom: h as i32,
                        },
                        default_color: Color::rgb(255, 255, 255),
                        custom_glyphs: &[],
                    },
                    TextArea {
                        buffer: buffer_xyzrpy,
                        left: w as f32 * (1.0 - self.scale) - 10.0,
                        top: h as f32 - 10.0 - LINE_HEIGHT * xyz_lines,
                        scale: 1.0,
                        bounds: TextBounds {
                            left: 0,
                            top: 0,
                            right: w as i32,
                            bottom: h as i32,
                        },
                        default_color: Color::rgb(255, 255, 255),
                        custom_glyphs: &[],
                    },
                ],
                swash_cache,
            )
            .context(PrepareSnafu)?;

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("performance/pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });
            renderer
                .render(atlas, viewport, &mut pass)
                .context(RenderSnafu)?;
        }

        atlas.trim();
        Ok(())
    }
}
