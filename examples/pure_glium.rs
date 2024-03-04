//! Example how to use `egui_glium`.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use egui::ViewportId;
use glium::{backend::glutin::SimpleWindowBuilder, glutin::surface::WindowSurface};
use winit::{
    event,
    event_loop::{EventLoop, EventLoopBuilder},
};

fn main() {
    let event_loop = EventLoopBuilder::with_user_event().build().unwrap();
    let (window, display) = create_display(&event_loop);

    let mut egui_glium =
        egui_glium::EguiGlium::new(ViewportId::ROOT, &display, &window, &event_loop);

    let mut color_test = egui_demo_lib::ColorTest::default();

    let result = event_loop.run(move |event, target| {
        let mut redraw = || {
            let mut quit = false;

            egui_glium.run(&window, |egui_ctx| {
                egui::SidePanel::left("my_side_panel").show(egui_ctx, |ui| {
                    ui.heading("Hello World!");
                    if ui.button("Quit").clicked() {
                        quit = true;
                    }
                });

                egui::CentralPanel::default().show(egui_ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        color_test.ui(ui);
                    });
                });
            });

            if quit {
                target.exit()
            }

            {
                use glium::Surface as _;
                let mut target = display.draw();

                let color = egui::Rgba::from_rgb(0.1, 0.3, 0.2);
                target.clear_color(color[0], color[1], color[2], color[3]);

                // draw things behind egui here

                egui_glium.paint(&display, &mut target);

                // draw things on top of egui here

                target.finish().unwrap();
            }
        };

        match event {
            event::Event::WindowEvent { event, .. } => {
                use event::WindowEvent;
                match &event {
                    WindowEvent::CloseRequested | WindowEvent::Destroyed => target.exit(),
                    WindowEvent::Resized(new_size) => {
                        display.resize((*new_size).into());
                    }
                    WindowEvent::RedrawRequested => redraw(),
                    _ => {}
                }

                let event_response = egui_glium.on_event(&window, &event);

                if event_response.repaint {
                    window.request_redraw();
                }
            }
            event::Event::NewEvents(event::StartCause::ResumeTimeReached { .. }) => {
                window.request_redraw();
            }
            _ => (),
        }
    });
    result.unwrap()
}

fn create_display(
    event_loop: &EventLoop<()>,
) -> (winit::window::Window, glium::Display<WindowSurface>) {
    SimpleWindowBuilder::new()
        .set_window_builder(winit::window::WindowBuilder::new().with_resizable(true))
        .with_inner_size(800, 600)
        .with_title("egui_glium example")
        .build(event_loop)
}
