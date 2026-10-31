use std::sync::atomic::{AtomicU32, Ordering::Relaxed};
enum Channel {
    Red,
    Green,
    Blue,
}

fn compute_channel_avg<const S: usize>(pixels: &[[AtomicU32; S]; S], channel: Channel) -> u8 {
    let offset: usize = match channel {
        Channel::Red => 16,
        Channel::Green => 8,
        Channel::Blue => 0,
    };

    let sum: u64 = pixels
        .iter()
        .map(|row| {
            row.iter()
                .map(|p| ((p.load(Relaxed) >> offset) & 0x0000FF) as u64)
                .sum::<u64>()
        })
        .sum();
    let num_pixels = (S as u64).pow(2);

    (sum / num_pixels) as u8
}

pub(crate) fn transform<const S: usize>(mut pixels: [[AtomicU32; S]; S]) {
    let (avg_r, avg_g, avg_b) = (
        compute_channel_avg(&pixels, Channel::Red),
        compute_channel_avg(&pixels, Channel::Green),
        compute_channel_avg(&pixels, Channel::Blue),
    );

    let avg = (avg_r as u32) << 16 | (avg_g as u32) << 8 | (avg_b as u32);

    for row in &mut pixels {
        for p in row {
            p.swap(avg, Relaxed);
        }
    }
}
