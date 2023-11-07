#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use egui::load::SizedTexture;
use glium::{backend::glutin::SimpleWindowBuilder, glutin::surface::WindowSurface};
use winit::event_loop::{self, EventLoop, EventLoopBuilder};

fn main() {
    let event_loop = EventLoopBuilder::with_user_event().build();
    let (window, display) = create_display(&event_loop);

    let mut egui_glium = egui_glium::EguiGlium::new(&display, &window, &event_loop);

    let png_data = include_bytes!("rust-logo-256x256.png");
    let image = load_glium_image(png_data);
    let image_size = egui::vec2(image.width as f32, image.height as f32);
    // Load to gpu memory
    let glium_texture = glium::texture::SrgbTexture2d::new(&display, image).unwrap();
    // Allow us to share the texture with egui:
    let glium_texture = std::rc::Rc::new(glium_texture);
    // Allocate egui's texture id for GL texture
    let texture_id = egui_glium
        .painter
        .register_native_texture(glium_texture, Default::default());
    // Setup button image size for reasonable image size for button container.
    let button_image_size = egui::vec2(32_f32, 32_f32);

    event_loop.run(move |event, _, control_flow| {
        let mut redraw = || {
            let mut quit = false;

            let repaint_after = egui_glium.run(&window, |egui_ctx| {
                egui::SidePanel::left("my_side_panel").show(egui_ctx, |ui| {
                    if ui
                        .add(egui::Button::image_and_text(
                            (texture_id, button_image_size),
                            "Quit",
                        ))
                        .clicked()
                    {
                        quit = true;
                    }
                });
                egui::Window::new("NativeTextureDisplay").show(egui_ctx, |ui| {
                    ui.image(SizedTexture::new(texture_id, image_size));
                });
            });

            *control_flow = if quit {
                event_loop::ControlFlow::Exit
            } else if repaint_after.is_zero() {
                window.request_redraw();
                event_loop::ControlFlow::Poll
            } else if let Some(repaint_after_instant) =
                std::time::Instant::now().checked_add(repaint_after)
            {
                event_loop::ControlFlow::WaitUntil(repaint_after_instant)
            } else {
                event_loop::ControlFlow::Wait
            };

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
            // Platform-dependent event handlers to workaround a winit bug
            // See: https://github.com/rust-windowing/winit/issues/987
            // See: https://github.com/rust-windowing/winit/issues/1619
            winit::event::Event::RedrawEventsCleared if cfg!(target_os = "windows") => redraw(),
            winit::event::Event::RedrawRequested(_) if !cfg!(target_os = "windows") => redraw(),

            winit::event::Event::WindowEvent { event, .. } => {
                use winit::event::WindowEvent;
                match &event {
                    WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                        *control_flow = winit::event_loop::ControlFlow::Exit;
                    }
                    WindowEvent::Resized(new_size) => {
                        display.resize((*new_size).into());
                    }
                    _ => {}
                }

                let event_response = egui_glium.on_event(&event);

                if event_response.repaint {
                    window.request_redraw();
                }
            }

            winit::event::Event::NewEvents(winit::event::StartCause::ResumeTimeReached {
                ..
            }) => {
                window.request_redraw();
            }

            _ => (),
        }
    });
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

fn load_glium_image(png_data: &[u8]) -> glium::texture::RawImage2d<'_, u8> {
    // Load image using the image crate:
    let image = image::load_from_memory(png_data).unwrap().to_rgba8();
    let image_dimensions = image.dimensions();

    // Premultiply alpha:
    let pixels: Vec<_> = image
        .into_vec()
        .chunks_exact(4)
        .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
        .flat_map(|color| color.to_array())
        .collect();

    // Convert to glium image:
    glium::texture::RawImage2d::from_raw_rgba(pixels, image_dimensions)
}
