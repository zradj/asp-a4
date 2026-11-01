use std::sync::Arc;

use winit::event_loop::EventLoopProxy;

use crate::{averaging, picture::Picture, visualization::Event};

pub(crate) fn spawn_worker(pic: Arc<Picture>, square_size: usize, proxy: EventLoopProxy<Event>) {
    std::thread::spawn(move || {
        std::thread::scope(|s| {
            let p = proxy.clone();
            s.spawn(move || averaging::worker(pic, square_size, p));
        });
        let _ = proxy.send_event(Event::AllDone);
    });
}
