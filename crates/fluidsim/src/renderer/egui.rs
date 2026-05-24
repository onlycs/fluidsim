use egui::{Context, Shadow};
use egui_wgpu::{Renderer, RendererOptions, ScreenDescriptor};
use egui_winit::{EventResponse, State};
use wgpu::{CommandEncoder, TextureView};
use winit::{event::WindowEvent, window::Window};

use super::graphics::GraphicsContext;

pub struct UiRenderer {
    context: Context,
    state: State,
    renderer: Renderer,
}

impl UiRenderer {
    pub fn new(wgpu: &GraphicsContext) -> Self {
        let ctx = Context::default();
        let id = ctx.viewport_id();

        ctx.set_visuals(egui::Visuals {
            window_shadow: Shadow::NONE,
            ..egui::Visuals::dark()
        });

        let state = State::new(ctx.clone(), id, &wgpu.window, None, None, None);
        let renderer = Renderer::new(&wgpu.device, wgpu.config.format, RendererOptions::default());

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
        wgpu: &GraphicsContext,
        encoder: &mut CommandEncoder,
        surface_view: &TextureView,
        ui: impl FnMut(&mut egui::Ui),
    ) {
        self.context.set_pixels_per_point(1.0);

        let input = self.state.take_egui_input(&wgpu.window);
        let output = self.context.run_ui(input, ui);

        self.state
            .handle_platform_output(&wgpu.window, output.platform_output);

        let clips = self
            .context
            .tessellate(output.shapes, output.pixels_per_point);

        for (id, delta) in output.textures_delta.set {
            self.renderer
                .update_texture(&wgpu.device, &wgpu.queue, id, &delta);
        }

        let desc = ScreenDescriptor {
            size_in_pixels: [wgpu.config.width, wgpu.config.height],
            pixels_per_point: self.context.pixels_per_point(),
        };

        self.renderer
            .update_buffers(&wgpu.device, &wgpu.queue, encoder, &clips, &desc);

        let rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("egui render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: surface_view,
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

        self.renderer
            .render(&mut rpass.forget_lifetime(), &clips, &desc);

        for x in &output.textures_delta.free {
            self.renderer.free_texture(x);
        }
    }
}
