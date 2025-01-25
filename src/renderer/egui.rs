use std::ops::{Deref, DerefMut};

use crate::prelude::*;

use egui::{Button, Context, RichText, Shadow, Slider};
use egui_wgpu::{Renderer, ScreenDescriptor};
use egui_winit::{EventResponse, State};
use wgpu::{CommandEncoder, TextureView};
use winit::{event::WindowEvent, window::Window};

use super::WgpuData;

const TEXT_SIZE: f32 = 16.0;

pub struct EguiTranslator {
    context: Context,
    state: State,
    renderer: Renderer,
}

impl EguiTranslator {
    pub fn new(wgpu: &WgpuData) -> Self {
        let ctx = Context::default();
        let id = ctx.viewport_id();

        ctx.set_visuals(egui::Visuals {
            window_shadow: Shadow::NONE,
            ..egui::Visuals::dark()
        });

        let state = State::new(ctx.clone(), id, &wgpu.window, None, None, None);
        let renderer = Renderer::new(&wgpu.device, wgpu.config.format, None, 1, false);

        Self {
            context: ctx,
            state,
            renderer,
        }
    }

    pub fn event(&mut self, window: &Window, event: &WindowEvent) -> EventResponse {
        self.state.on_window_event(window, event)
    }

    pub fn draw(
        &mut self,
        wgpu: &WgpuData,
        mut encoder: &mut CommandEncoder,
        surface_view: &TextureView,
        ui: impl FnMut(&Context),
    ) {
        let input = self.state.take_egui_input(&*wgpu.window);
        let output = self.context.run(input, ui);

        self.state
            .handle_platform_output(&*wgpu.window, output.platform_output);

        let clips = self
            .context
            .tessellate(output.shapes, output.pixels_per_point);

        for (id, delta) in output.textures_delta.set {
            self.renderer
                .update_texture(&wgpu.device, &wgpu.queue, id, &delta);
        }

        let desc = ScreenDescriptor {
            size_in_pixels: [wgpu.config.width, wgpu.config.height],
            pixels_per_point: wgpu.window.scale_factor() as f32,
        };

        self.renderer
            .update_buffers(&wgpu.device, &wgpu.queue, &mut encoder, &clips, &desc);

        let rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("egui render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: surface_view,
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

        self.renderer
            .render(&mut rpass.forget_lifetime(), &clips, &desc);

        for x in &output.textures_delta.free {
            self.renderer.free_texture(x)
        }
    }
}

#[derive(Default)]
pub struct EguiState(Option<EguiTranslator>);

impl EguiState {
    pub fn uninit(&self) -> bool {
        self.0.is_none()
    }

    pub fn init(&mut self, wgpu: &WgpuData) {
        self.0 = Some(EguiTranslator::new(wgpu));
    }
}

impl Deref for EguiState {
    type Target = EguiTranslator;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl DerefMut for EguiState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap()
    }
}
