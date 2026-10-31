use image::Rgb;

enum Channel {
    Red,
    Green,
    Blue,
}

fn compute_channel_avg<const S: usize>(pixels: [[Rgb<u8>; S]; S], channel: Channel) -> u8 {
    let index: usize = match channel {
        Channel::Red => 0,
        Channel::Green => 1,
        Channel::Blue => 2,
    };

    let sum: u64 = pixels
        .iter()
        .map(|row| row.iter().map(|p| p[index] as u64).sum::<u64>())
        .sum();
    let num_pixels = (S as u64).pow(2);

    (sum / num_pixels) as u8
}

pub(crate) fn transform<const S: usize>(mut pixels: [[Rgb<u8>; S]; S]) {
    let (avg_r, avg_g, avg_b) = (
        compute_channel_avg(pixels, Channel::Red),
        compute_channel_avg(pixels, Channel::Green),
        compute_channel_avg(pixels, Channel::Blue),
    );

    for row in &mut pixels {
        for rgb in row {
            rgb.0 = [avg_r, avg_g, avg_b];
        }
    }
}
