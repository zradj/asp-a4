use std::{
    fs::File,
    io::BufWriter,
    num::NonZeroU32,
    rc::Rc,
    sync::{Arc, atomic::Ordering::Relaxed},
};

use image::RgbImage;
use rdev::display_size;
use softbuffer::{Context, Surface};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop, EventLoopProxy, OwnedDisplayHandle},
    window::Window,
};

use crate::picture::Picture;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Event {
    IterationDone,
    AllDone,
}

struct App {
    context: Context<OwnedDisplayHandle>,
    window: Option<Rc<Window>>,
    surface: Option<Surface<OwnedDisplayHandle, Rc<Window>>>,
    picture: Arc<Picture>,
    window_w: u32,
    window_h: u32,
}

impl ApplicationHandler<Event> for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let mut window_attrs = Window::default_attributes()
            .with_title("Image")
            .with_inner_size(PhysicalSize::new(self.window_w, self.window_h))
            .with_decorations(false)
            .with_position(PhysicalPosition::new(self.window_w / 2, self.window_h / 2));
        window_attrs.resizable = false;
        if let Ok((w, h)) = display_size() {
            let x = (w.saturating_sub(self.window_w as u64)) / 2;
            let y = (h.saturating_sub(self.window_h as u64)) / 2;

            window_attrs = window_attrs.with_position(PhysicalPosition::new(x as i32, y as i32));
        }
        let window = self
            .window
            .get_or_insert_with(|| {
                Rc::new(
                    event_loop
                        .create_window(window_attrs)
                        .expect("Failed to create a window for the image"),
                )
            })
            .clone();
        if self.surface.is_none() {
            self.surface = Some(Surface::new(&self.context, window).unwrap());
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                let (Some(window), Some(surface)) = (&self.window, &mut self.surface) else {
                    return;
                };

                let size = window.inner_size();
                let (Some(width), Some(height)) =
                    (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                else {
                    return;
                };
                let (bw, bh) = (width.get(), height.get());
                surface
                    .resize(width, height)
                    .expect("Failed to resize the window");

                let mut buffer = surface.buffer_mut().unwrap();
                let (pw, ph) = (self.picture.width as u64, self.picture.height as u64);
                for y in 0..bh as u64 {
                    let sy = y * ph / bh as u64;
                    let row = &self.picture.pixels[(sy * pw) as usize..][..pw as usize];
                    for x in 0..bw as u64 {
                        let sx = x * pw / bw as u64;
                        buffer[(y * bw as u64 + x) as usize] = row[sx as usize].load(Relaxed);
                    }
                }
                buffer.present().expect("Failed to update the window");
            }
            _ => {}
        }
    }

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: Event) {
        match event {
            Event::IterationDone => {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            Event::AllDone => {
                let rgb: Vec<u8> = self
                    .picture
                    .pixels
                    .iter()
                    .flat_map(|p| {
                        let p = p.load(Relaxed);
                        [(p >> 16) as u8, (p >> 8) as u8, p as u8]
                    })
                    .collect();
                let img = RgbImage::from_raw(self.picture.width, self.picture.height, rgb).unwrap();
                let output_file =
                    File::create("result.jpg").expect("Could not create or truncate result.jpg");
                img.write_to(&mut BufWriter::new(output_file), image::ImageFormat::Jpeg)
                    .expect("Could not write to result.jpg");
                event_loop.exit();
            }
        }
    }
}

pub(crate) fn visualize<F>(
    picture: Arc<Picture>,
    square_size: usize,
    window_w: u32,
    window_h: u32,
    spawn_workers: F,
) where
    F: FnOnce(Arc<Picture>, usize, EventLoopProxy<Event>),
{
    let event_loop = EventLoop::with_user_event().build().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);
    let proxy = event_loop.create_proxy();

    spawn_workers(picture.clone(), square_size, proxy.clone());

    let context = softbuffer::Context::new(event_loop.owned_display_handle()).unwrap();
    let mut app = App {
        context,
        window: None,
        surface: None,
        picture,
        window_w,
        window_h,
    };

    event_loop.run_app(&mut app).unwrap();
}
