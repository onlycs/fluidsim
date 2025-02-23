use std::{
    io,
    ops::{Deref, DerefMut},
};

use glyphon::{
    Attrs, Buffer, Cache, Color, FontSystem, Metrics, Resolution, SwashCache, TextArea, TextAtlas,
    TextBounds, TextRenderer, Viewport,
};
use wgpu::{CommandEncoder, MultisampleState, TextureView};

use super::wgpu_state::WgpuData;
use crate::prelude::*;

const FONT_SIZE: f32 = 18.;
const LINE_HEIGHT: f32 = 24.;

pub struct FpsCounter {
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    pub viewport: Viewport,
    pub atlas: TextAtlas,
    pub renderer: TextRenderer,
    pub buffer: Buffer,

    pub timer: Instant,
    pub frames: u32,
    pub last_fps: f32,
}

impl FpsCounter {
    pub fn new(wgpu: &WgpuData) -> Result<Self, io::Error> {
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

        Ok(Self {
            font_system,
            swash_cache,
            viewport,
            atlas: text_altas,
            renderer,
            buffer,

            timer: Instant::now(),
            frames: 0,
            last_fps: 0.,
        })
    }

    pub fn update(&mut self) {
        self.frames += 1;

        if self.timer.elapsed().as_secs_f32() > 1. {
            self.last_fps = self.frames as f32 / self.timer.elapsed().as_secs_f32();
            self.frames = 0;
            self.timer = Instant::now();
        }
    }

    pub fn render(
        &mut self,
        wgpu: &WgpuData,
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

        let WgpuData {
            device,
            queue,
            config,
            ..
        } = wgpu;

        buffer.set_text(
            font_system,
            format!("FPS: {:.2}", self.last_fps).as_str(),
            Attrs::new().family(glyphon::Family::Name("JetBrains Mono")),
            glyphon::Shaping::Advanced,
        );

        buffer.shape_until_scroll(font_system, false);

        viewport.update(
            queue,
            Resolution {
                width: config.width,
                height: config.height,
            },
        );

        renderer.prepare(
            device,
            queue,
            font_system,
            atlas,
            viewport,
            [TextArea {
                buffer,
                left: 10.,
                top: config.height as f32 - 10. - LINE_HEIGHT,
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
        )?;

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
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            renderer.render(atlas, viewport, &mut pass)?;
        }

        atlas.trim();

        Ok(())
    }
}

#[derive(Default)]
pub struct FpsState(Option<FpsCounter>);

impl FpsState {
    pub fn uninit(&self) -> bool {
        self.0.is_none()
    }

    pub fn init(&mut self, wgpu: &WgpuData) -> Result<(), io::Error> {
        self.0 = Some(FpsCounter::new(wgpu)?);

        Ok(())
    }
}

impl Deref for FpsState {
    type Target = FpsCounter;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl DerefMut for FpsState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap()
    }
}
