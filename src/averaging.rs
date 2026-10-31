use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering::Relaxed},
};

use winit::event_loop::EventLoopProxy;

use crate::visualization;

pub(crate) fn worker(
    pixels: Arc<Vec<AtomicU32>>,
    square_size: usize,
    width: u32,
    height: u32,
    proxy: EventLoopProxy<visualization::Event>,
) {
    let max_done_x = (width as usize).div_ceil(square_size);
    let max_done_y = (height as usize).div_ceil(square_size);
    let mut done_x = 0;
    let mut done_y = 0;
    // work for 10 seconds regardless of the square size
    let sleep_duration =
        std::time::Duration::from_nanos(10000000000 / ((max_done_x as u64) * (max_done_y as u64)));

    while done_y < max_done_y {
        let (mut sum_r, mut sum_g, mut sum_b): (u64, u64, u64) = (0, 0, 0);
        let mut px_cnt: u64 = 0;

        for px_row in pixels
            .chunks(width as usize)
            .skip(done_y * square_size)
            .take(square_size)
        {
            for px_col in px_row.chunks(square_size).skip(done_x).take(1) {
                for px in px_col {
                    let px = px.load(Relaxed) as u64;
                    sum_r += (px >> 16) & 0x0000FF;
                    sum_g += (px >> 8) & 0x0000FF;
                    sum_b += px & 0x0000FF;
                    px_cnt += 1;
                }
            }
        }

        let (avg_r, avg_g, avg_b) = (sum_r / px_cnt, sum_g / px_cnt, sum_b / px_cnt);
        let avg = (avg_r as u32) << 16 | (avg_g as u32) << 8 | (avg_b as u32);

        for px_row in pixels
            .chunks(width as usize)
            .skip(done_y * square_size)
            .take(square_size)
        {
            for px_col in px_row.chunks(square_size).skip(done_x).take(1) {
                for px in px_col {
                    px.store(avg, Relaxed);
                }
            }
        }

        if done_x + 1 < max_done_x {
            done_x += 1;
        } else {
            done_x = 0;
            done_y += 1;
        }

        if proxy
            .send_event(visualization::Event::IterationDone)
            .is_err()
        {
            return;
        }

        std::thread::sleep(sleep_duration);
    }
}
