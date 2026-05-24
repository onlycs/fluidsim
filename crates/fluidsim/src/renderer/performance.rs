use std::sync::Mutex;

use glam::UVec2;
use glyphon::{
    Attrs, Buffer, Cache, Color, FontSystem, Metrics, Resolution, SwashCache, TextArea, TextAtlas,
    TextBounds, TextRenderer, Viewport, Weight,
};
use wgpu::{CommandEncoder, MultisampleState, TextureView};

use super::{graphics::GraphicsContext, shader::pipelines::ComputeShaderPerformance};
use crate::prelude::*;

const FONT_SIZE: f32 = 18.;
const LINE_HEIGHT: f32 = 24.;

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
    pub(crate) font_system: FontSystem,
    pub(crate) swash_cache: SwashCache,
    pub(crate) viewport: Viewport,
    pub(crate) atlas: TextAtlas,
    pub(crate) renderer: TextRenderer,
    pub(crate) buffer: Buffer,

    pub(crate) data: Arc<Mutex<ComputeShaderPerformance>>,
    pub(crate) last_data: ComputeShaderPerformance,
    pub(crate) show: bool,

    pub(crate) timer: Instant,
    pub(crate) frames: usize,
    pub(crate) fps: f32,
}

impl PerformanceDisplay {
    pub(crate) fn new(wgpu: &GraphicsContext) -> Self {
        let size = wgpu.window.inner_size();
        let scale = wgpu.window.scale_factor() as f32;

        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(&wgpu.device);
        let viewport = Viewport::new(&wgpu.device, &cache);
        let mut text_altas = TextAtlas::new(&wgpu.device, &wgpu.queue, &cache, wgpu.config.format);
        let renderer = TextRenderer::new(
            &mut text_altas,
            &wgpu.device,
            MultisampleState::default(),
            None,
        );
        let mut buffer = Buffer::new(&mut font_system, Metrics::new(FONT_SIZE, LINE_HEIGHT));

        buffer.set_size(
            &mut font_system,
            Some(size.width as f32 * scale),
            Some(size.height as f32 * scale),
        );

        // wasm has no fonts according to cosmic, but include anyways cuz i like jbm
        font_system.db_mut().load_font_data(
            include_bytes!(concat!(
                env!("CARGO_WORKSPACE_DIR"),
                "assets/font/JetBrainsMono-Light.ttf"
            ))
            .to_vec(),
        );

        Self {
            font_system,
            swash_cache,
            viewport,
            atlas: text_altas,
            renderer,
            buffer,

            show: false,
            data: Arc::new(Mutex::new(ComputeShaderPerformance::default())),
            last_data: ComputeShaderPerformance::default(),

            timer: Instant::now(),
            frames: 0,
            fps: 0.,
        }
    }

    pub(crate) fn update(&mut self) {
        self.frames += 1;

        if self.timer.elapsed().as_secs_f32() > 1. {
            self.fps = self.frames as f32 / self.timer.elapsed().as_secs_f32();
            self.frames = 0;
            self.last_data = *self.data.lock().unwrap();
            self.timer = Instant::now();
        }
    }

    pub(crate) fn resize(&mut self, size: UVec2, scale: f32) {
        self.buffer.set_size(
            &mut self.font_system,
            Some(size.x as f32 * scale),
            Some(size.y as f32 * scale),
        );
    }

    pub(crate) fn toggle(&mut self) {
        self.show = !self.show;
    }

    pub(crate) fn render(
        &mut self,
        wgpu: &GraphicsContext,
        encoder: &mut CommandEncoder,
        view: &TextureView,
    ) -> Result<(), TextError> {
        let Self {
            font_system,
            swash_cache,
            viewport,
            atlas,
            renderer,
            buffer,
            ..
        } = self;

        let GraphicsContext {
            device,
            queue,
            config,
            ..
        } = wgpu;

        let mut text = format!("FPS: {:.2}", self.fps);

        if self.show {
            text = format!("{}{text}", self.last_data);
        }

        buffer.set_text(
            font_system,
            text.as_str(),
            &Attrs::new()
                .family(glyphon::Family::Name("JetBrains Mono"))
                .weight(Weight::LIGHT),
            glyphon::Shaping::Advanced,
            None,
        );

        buffer.shape_until_scroll(font_system, false);

        viewport.update(
            queue,
            Resolution {
                width: config.width,
                height: config.height,
            },
        );

        // RP1: draw fps
        renderer
            .prepare(
                device,
                queue,
                font_system,
                atlas,
                viewport,
                [TextArea {
                    buffer,
                    left: 10.,
                    top: config.height as f32
                        - 10.
                        - (LINE_HEIGHT * (text.split('\n').count() as f32)),
                    scale: 1.,
                    bounds: TextBounds {
                        left: 0,
                        top: 0,
                        right: config.width as i32,
                        bottom: config.height as i32,
                    },
                    default_color: Color::rgb(255, 255, 255),
                    custom_glyphs: &[],
                }],
                swash_cache,
            )
            .context(PrepareSnafu)?;

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("text render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            renderer
                .render(atlas, viewport, &mut pass)
                .context(RenderSnafu)?;
        }

        atlas.trim();

        Ok(())
    }
}
