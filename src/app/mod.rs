pub mod error;
pub mod state;

use crate::prelude::*;
use state::State;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowId},
};

pub struct App {
    window: Option<Arc<Window>>,
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        info!("Creating window");

        if self.window.is_some() {
            return;
        }

        let window = Arc::new(
            event_loop
                .create_window(WindowAttributes::default().with_title("titleitle"))
                .unwrap(),
        );

        let state = task::block_on(State::new(Arc::clone(&window))).unwrap();

        self.window = Some(Arc::clone(&window));
        self.state = Some(state);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let (Some(window), Some(state)) =
            (self.window.as_ref().map(Arc::clone), self.state.as_mut())
        else {
            return;
        };

        if state.handle(&event) {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                info!("Window close requested");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                state.window().request_redraw();
                state.update();

                trace!("Redraw requested");

                match state.render() {
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        debug!("Need to resize");
                        state.resize(window.inner_size());
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        error!("Out of memory");
                        event_loop.exit();
                    }
                    Err(wgpu::SurfaceError::Timeout) => {
                        warn!("Timeout")
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

impl Default for App {
    #[allow(invalid_value)]
    fn default() -> Self {
        Self {
            window: None,
            state: None,
        }
    }
}
