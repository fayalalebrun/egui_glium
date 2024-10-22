//! [`egui`] bindings for [`glium`](https://github.com/glium/glium).
//!
//! The main type you want to use is [`EguiGlium`].
//!
//! If you are writing an app, you may want to look at [`eframe`](https://docs.rs/eframe) instead.
//!
//! ## Feature flags
#![cfg_attr(feature = "document-features", doc = document_features::document_features!())]
//!

#![allow(clippy::float_cmp)]
#![allow(clippy::manual_range_contains)]
#![forbid(unsafe_code)]

mod painter;
use glium::glutin::surface::WindowSurface;
pub use painter::Painter;

pub use egui_winit;

use egui_winit::winit::event_loop::ActiveEventLoop;
pub use egui_winit::EventResponse;

// ----------------------------------------------------------------------------

/// Convenience wrapper for using [`egui`] from a [`glium`] app.
pub struct EguiGlium {
    pub egui_winit: egui_winit::State,
    pub painter: crate::Painter,

    shapes: Vec<egui::epaint::ClippedShape>,
    textures_delta: egui::TexturesDelta,
}

impl EguiGlium {
    pub fn new(
        viewport_id: egui::ViewportId,
        display: &glium::Display<WindowSurface>,
        window: &winit::window::Window,
        event_loop: &ActiveEventLoop,
    ) -> Self {
        let painter = crate::Painter::new(display);

        let pixels_per_point = window.scale_factor() as f32;
        let egui_winit = egui_winit::State::new(
            Default::default(),
            viewport_id,
            event_loop,
            Some(pixels_per_point),
            None,
            Some(painter.max_texture_side()),
        );

        Self {
            egui_winit,
            painter,
            shapes: Default::default(),
            textures_delta: Default::default(),
        }
    }

    pub fn egui_ctx(&self) -> &egui::Context {
        self.egui_winit.egui_ctx()
    }

    pub fn on_event(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::WindowEvent,
    ) -> EventResponse {
        self.egui_winit.on_window_event(window, event)
    }

    /// Runs the main egui render.
    ///
    /// Call [`Self::paint`] later to paint.
    pub fn run(&mut self, window: &winit::window::Window, run_ui: impl FnMut(&egui::Context)) {
        let raw_input = self.egui_winit.take_egui_input(window);
        let egui::FullOutput {
            platform_output,
            textures_delta,
            shapes,
            ..
        } = self.egui_ctx().run(raw_input, run_ui);

        self.egui_winit
            .handle_platform_output(window, platform_output);

        self.shapes = shapes;
        self.textures_delta.append(textures_delta);
    }

    /// Paint the results of the last call to [`Self::run`].
    pub fn paint<T: glium::Surface>(
        &mut self,
        display: &glium::Display<WindowSurface>,
        target: &mut T,
    ) {
        let shapes = std::mem::take(&mut self.shapes);
        let textures_delta = std::mem::take(&mut self.textures_delta);
        let clipped_primitives = self
            .egui_ctx()
            .tessellate(shapes, self.egui_ctx().pixels_per_point());
        self.painter.paint_and_update_textures(
            display,
            target,
            self.egui_ctx().pixels_per_point(),
            &clipped_primitives,
            &textures_delta,
        );
    }
}
