#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use egui::{load::SizedTexture, TextureId, Vec2, ViewportId};
use glium::{
    backend::glutin::SimpleWindowBuilder, glutin::surface::WindowSurface, texture::SrgbTexture2d,
};
use std::rc::Rc;
use winit::{
    application::ApplicationHandler,
    event::{StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

fn main() {
    let event_loop = EventLoop::<()>::with_user_event().build().unwrap();

    let (window, glium_display) = SimpleWindowBuilder::new()
        .set_window_builder(Window::default_attributes().with_resizable(true))
        .with_inner_size(800, 600)
        .with_title("egui_glium example")
        .build(&event_loop);

    let mut egui_glium_instance =
        egui_glium::EguiGlium::new(ViewportId::ROOT, &glium_display, &window, &event_loop);

    let png_data = include_bytes!("rust-logo-256x256.png");
    let image = load_glium_image(png_data);
    let image_size = egui::vec2(image.width as f32, image.height as f32);
    // Load to gpu memory
    let glium_texture = glium::texture::SrgbTexture2d::new(&glium_display, image).unwrap();
    // Allow us to share the texture with egui:
    let glium_texture = std::rc::Rc::new(glium_texture);
    // Allocate egui's texture id for GL texture
    let texture_id = egui_glium_instance
        .painter
        .register_native_texture(Rc::clone(&glium_texture), Default::default());

    // Setup button image size for reasonable image size for button container.
    let button_image_size = egui::vec2(32_f32, 32_f32);

    let mut app = App {
        egui_glium_instance,
        _glium_texture: glium_texture,
        texture_id,
        image_size,
        window,
        display: glium_display,
        button_image_size,
    };

    let result = event_loop.run_app(&mut app);
    result.unwrap()
}

struct App {
    egui_glium_instance: egui_glium::EguiGlium,
    _glium_texture: Rc<SrgbTexture2d>,
    texture_id: TextureId,
    image_size: Vec2,
    window: winit::window::Window,
    display: glium::Display<WindowSurface>,
    button_image_size: Vec2,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let mut redraw = || {
            let mut quit = false;

            self.egui_glium_instance.run(&self.window, |egui_ctx| {
                egui::SidePanel::left("my_side_panel").show(egui_ctx, |ui| {
                    if ui
                        .add(egui::Button::image_and_text(
                            (self.texture_id, self.button_image_size),
                            "Quit",
                        ))
                        .clicked()
                    {
                        quit = true;
                    }
                });
                egui::Window::new("NativeTextureDisplay").show(egui_ctx, |ui| {
                    ui.image(SizedTexture::new(self.texture_id, self.image_size));
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

                self.egui_glium_instance.paint(&self.display, &mut target);

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

        let event_response = self.egui_glium_instance.on_event(&self.window, &event);

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
