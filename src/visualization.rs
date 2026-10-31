use std::{
    num::NonZeroU32,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering::Relaxed},
    },
};

use softbuffer::{Context, Surface};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop, OwnedDisplayHandle},
    window::Window,
};

use crate::averaging;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Event {
    IterationDone,
}

struct App {
    context: Context<OwnedDisplayHandle>,
    window: Option<Rc<Window>>,
    surface: Option<Surface<OwnedDisplayHandle, Rc<Window>>>,
    image: Arc<Vec<AtomicU32>>,
    width: u32,
    height: u32,
}

impl ApplicationHandler<Event> for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let mut window_attrs = Window::default_attributes()
            .with_inner_size(PhysicalSize::new(self.width, self.height))
            .with_decorations(false);
        window_attrs.resizable = false;
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
                surface
                    .resize(width, height)
                    .expect("Failed to resize the window");

                let mut buffer = surface.buffer_mut().unwrap();
                buffer.fill(0);
                let (bw, bh) = (width.get(), height.get());
                for y in 0..self.height.min(bh) {
                    for x in 0..self.width.min(bw) {
                        buffer[(y * bw + x) as usize] =
                            self.image[(y * self.width + x) as usize].load(Relaxed);
                    }
                }
                buffer.present().expect("Failed to update the window");
            }
            _ => {}
        }
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: Event) {
        match event {
            Event::IterationDone => {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
        }
    }
}

pub(crate) fn visualize(width: u32, height: u32, image: Arc<Vec<AtomicU32>>, square_size: usize) {
    let event_loop = EventLoop::with_user_event().build().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);
    let proxy = event_loop.create_proxy();
    let worker_image = image.clone();

    std::thread::spawn(move || averaging::worker(worker_image, square_size, width, height, proxy));

    let context = softbuffer::Context::new(event_loop.owned_display_handle()).unwrap();
    let mut app = App {
        context,
        window: None,
        surface: None,
        image,
        width,
        height,
    };

    event_loop.run_app(&mut app).unwrap();
}
