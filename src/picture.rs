use std::sync::atomic::AtomicU32;

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
}
