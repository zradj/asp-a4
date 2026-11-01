use std::sync::Arc;

use winit::event_loop::EventLoopProxy;

use crate::{averaging, picture::Picture, single, visualization::Event};

pub(crate) fn spawn_workers(pic: Arc<Picture>, square_size: usize, proxy: EventLoopProxy<Event>) {
    let Some(core_ids) = core_affinity::get_core_ids() else {
        eprintln!("Warning: Failed to obtain core IDs, switching to the single threaded mode");
        single::spawn_worker(pic, square_size, proxy);
        return;
    };
    let num_cores = core_ids.len();
    println!("Info: There are {num_cores} cores present");

    std::thread::spawn(move || {
        std::thread::scope(|s| {
            let total_rows = (pic.height as usize).div_ceil(square_size);
            let rows_per_thread = total_rows.div_ceil(num_cores);
            let mut remaining_rows = total_rows;
            for (i, core_id) in core_ids.into_iter().enumerate() {
                let pic = pic.clone();
                let proxy = proxy.clone();
                let assigned_rows = rows_per_thread.min(remaining_rows);
                remaining_rows -= assigned_rows;
                s.spawn(move || {
                    let res = core_affinity::set_for_current(core_id);
                    if res {
                        averaging::worker(
                            pic,
                            square_size,
                            proxy,
                            i * rows_per_thread,
                            assigned_rows,
                        );
                    }
                });
            }
        });
        let _ = proxy.send_event(Event::AllDone);
    });
}
