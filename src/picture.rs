use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

use image::{DynamicImage, RgbImage};

pub(crate) struct Picture {
    pub(crate) pixels: Vec<AtomicU32>,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl Picture {
    pub(crate) fn new(pixels: Vec<AtomicU32>, width: u32, height: u32) -> Self {
        Self {
            pixels,
            width,
            height,
        }
    }

    pub(crate) fn get_raw(&self) -> Vec<u32> {
        self.pixels.iter().map(|p| p.load(Relaxed)).collect()
    }

    pub(crate) fn get_resized_raw(&self, new_width: u32, new_height: u32) -> Vec<u32> {
        let raw: Vec<u8> = self
            .pixels
            .iter()
            .flat_map(|p| {
                let p = p.load(Relaxed);
                [(p >> 16) as u8, (p >> 8) as u8, p as u8]
            })
            .collect();

        let img = RgbImage::from_raw(self.width, self.height, raw).unwrap();
        let img = DynamicImage::ImageRgb8(img).resize(
            new_width,
            new_height,
            image::imageops::FilterType::CatmullRom,
        );

        img.to_rgb8()
            .pixels()
            .map(|p| (p[0] as u32) << 16 | (p[1] as u32) << 8 | p[2] as u32)
            .collect()
    }
}
