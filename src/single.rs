use std::sync::Arc;

use winit::event_loop::EventLoopProxy;

use crate::{averaging, picture::Picture, visualization::Event};

pub(crate) fn spawn_worker(pic: Arc<Picture>, square_size: usize, proxy: EventLoopProxy<Event>) {
    std::thread::spawn(move || {
        std::thread::scope(|s| {
            let p = proxy.clone();
            s.spawn(move || {
                let total_rows = (pic.height as usize).div_ceil(square_size);
                averaging::worker(pic, square_size, p, 0, total_rows);
            });
        });
        let _ = proxy.send_event(Event::AllDone);
    });
}
