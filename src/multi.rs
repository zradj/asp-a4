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
            let num_threads = total_rows.min(num_cores);
            let (base, extra) = (total_rows / num_threads, total_rows % num_threads);

            if num_threads < num_cores {
                println!(
                    "Info: Only {num_threads} cores will be used - the square size is too large"
                );
            }

            let mut offset = 0;
            for (i, core_id) in core_ids.into_iter().take(num_threads).enumerate() {
                let pic = pic.clone();
                let proxy = proxy.clone();
                // add 1 to the assigned rows until the remainder is fully used
                let assigned_rows = base + usize::from(i < extra);
                let start = offset;
                offset += assigned_rows;
                s.spawn(move || {
                    let res = core_affinity::set_for_current(core_id);
                    if res {
                        averaging::worker(pic, square_size, proxy, start, assigned_rows);
                    }
                });
            }
        });
        let _ = proxy.send_event(Event::AllDone);
    });
}
