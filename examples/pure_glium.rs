//! Example how to use `egui_glium`.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use egui::ViewportId;
use glium::{backend::glutin::SimpleWindowBuilder, glutin::surface::WindowSurface};
use winit::{
    application::ApplicationHandler,
    event::{StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

fn main() {
    let event_loop = EventLoop::new().unwrap();

    let (window, display) = create_display(&event_loop);

    let egui_glium = egui_glium::EguiGlium::new(ViewportId::ROOT, &display, &window, &event_loop);

    let color_test = egui_demo_lib::ColorTest::default();

    let mut app = App {
        egui_glium,
        window,
        display,
        color_test,
    };

    let result = event_loop.run_app(&mut app);
    result.unwrap()
}

struct App {
    egui_glium: egui_glium::EguiGlium,
    window: winit::window::Window,
    display: glium::Display<WindowSurface>,
    color_test: egui_demo_lib::ColorTest,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let mut redraw = || {
            let mut quit = false;

            self.egui_glium.run(&self.window, |egui_ctx| {
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

            if quit {
                event_loop.exit()
            }

            {
                use glium::Surface as _;
                let mut target = self.display.draw();

                let color = egui::Rgba::from_rgb(0.1, 0.3, 0.2);
                target.clear_color(color[0], color[1], color[2], color[3]);

                // draw things behind egui here

                self.egui_glium.paint(&self.display, &mut target);

                // draw things on top of egui here

                target.finish().unwrap();
            }
        };

        use winit::event::WindowEvent;
        match &event {
            WindowEvent::CloseRequested | WindowEvent::Destroyed => event_loop.exit(),
            WindowEvent::Resized(new_size) => {
                self.display.resize((*new_size).into());
            }
            WindowEvent::RedrawRequested => redraw(),
            _ => {}
        }

        let event_response = self.egui_glium.on_event(&self.window, &event);

        if event_response.repaint {
            self.window.request_redraw();
        }
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        if let StartCause::ResumeTimeReached { .. } = cause {
            self.window.request_redraw();
        }
    }
}

fn create_display(
    event_loop: &EventLoop<()>,
) -> (winit::window::Window, glium::Display<WindowSurface>) {
    SimpleWindowBuilder::new()
        .set_window_builder(Window::default_attributes().with_resizable(true))
        .with_inner_size(800, 600)
        .with_title("egui_glium example")
        .build(event_loop)
}
