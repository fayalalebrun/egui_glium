//! Example how to use `egui_glium`.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use crate::event::WindowEvent;
use egui::ViewportId;
use glium::{backend::glutin::SimpleWindowBuilder, glutin::surface::WindowSurface};
use winit::{
    application::ApplicationHandler,
    event::{self, StartCause},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

fn main() {
    let event_loop = EventLoop::<()>::with_user_event().build().unwrap();

    let color_test = egui_demo_lib::ColorTest::default();

    let mut app = App {
        graphics_context: None,
        color_test,
    };

    let result = event_loop.run_app(&mut app);
    result.unwrap()
}

struct App {
    graphics_context: Option<GraphicsContext>,
    color_test: egui_demo_lib::ColorTest,
}

struct GraphicsContext {
    egui_glium_instance: egui_glium::EguiGlium,
    window: winit::window::Window,
    display: glium::Display<WindowSurface>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let (window, glium_display) = SimpleWindowBuilder::new()
            .set_window_builder(Window::default_attributes().with_resizable(true))
            .with_inner_size(800, 600)
            .with_title("egui_glium example")
            .build(event_loop);

        let egui_glium_instance =
            egui_glium::EguiGlium::new(ViewportId::ROOT, &glium_display, &window, event_loop);

        self.graphics_context = Some(GraphicsContext {
            egui_glium_instance,
            window,
            display: glium_display,
        });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let mut redraw = || {
            let mut quit = false;

            if let Some(graphics_context) = &mut self.graphics_context {
                graphics_context
                    .egui_glium_instance
                    .run(&graphics_context.window, |egui_ctx| {
                        egui::SidePanel::left("my_side_panel").show(egui_ctx, |ui| {
                            ui.heading("Hello World!");
                            if ui.button("Quit").clicked() {
                                quit = true;
                            }
                        });

                        egui::CentralPanel::default().show(egui_ctx, |ui| {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                self.color_test.ui(ui);
                            });
                        });
                    });
            }

            if quit {
                event_loop.exit()
            }

            if let Some(context) = &mut self.graphics_context {
                use glium::Surface as _;
                let mut target = context.display.draw();

                let color = egui::Rgba::from_rgb(0.1, 0.3, 0.2);
                target.clear_color(color[0], color[1], color[2], color[3]);

                // draw things behind egui here

                context
                    .egui_glium_instance
                    .paint(&context.display, &mut target);

                // draw things on top of egui here

                target.finish().unwrap();
            }
        };

        use event::WindowEvent;
        match &event {
            WindowEvent::CloseRequested | WindowEvent::Destroyed => event_loop.exit(),
            WindowEvent::Resized(new_size) => {
                if let Some(graphics_context) = &mut self.graphics_context {
                    graphics_context.display.resize((*new_size).into());
                }
            }
            WindowEvent::RedrawRequested => redraw(),
            _ => {}
        }

        if let Some(graphics_context) = &mut self.graphics_context {
            let event_response = graphics_context
                .egui_glium_instance
                .on_event(&graphics_context.window, &event);

            if event_response.repaint {
                graphics_context.window.request_redraw();
            }
        }
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        if let StartCause::ResumeTimeReached { .. } = cause {
            if let Some(graphics_context) = &mut self.graphics_context {
                graphics_context.window.request_redraw();
            }
        }
    }
}
