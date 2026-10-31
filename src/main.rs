mod averaging;
mod single;
mod visualization;

use std::{
    env::{self, Args},
    fmt::Display,
    fs::File,
    io::{self, BufReader},
    num::NonZeroUsize,
    process,
    sync::{Arc, atomic::AtomicU32},
};

use image::{ImageFormat, ImageReader};
use rdev::display_size;

use crate::visualization::visualize;

fn fail(msg: impl Display) -> ! {
    eprintln!("{msg}");
    process::exit(1);
}

#[derive(Debug)]
enum ProcessingMode {
    Single,
    Multi,
}

struct Settings {
    file_name: String,
    square_size: NonZeroUsize,
    mode: ProcessingMode,
}

impl From<Args> for Settings {
    fn from(args: Args) -> Self {
        let args: Vec<String> = args.collect();
        let Some([_, file_name, square_size, mode]) = args.as_array() else {
            let name = args.first().map_or("program", String::as_str);
            fail(format!(
                "Usage: {name} <filename> <square size> <processing mode>",
            ));
        };

        let square_size = square_size
            .parse()
            .unwrap_or_else(|_| fail("Error: Square size must be a positive integer"));

        let mode = match mode.to_ascii_uppercase().as_str() {
            "S" => ProcessingMode::Single,
            "M" => ProcessingMode::Multi,
            _ => fail(
                "Error: The processing mode must be either 'S' (single threaded) or 'M' (multi threaded)",
            ),
        };

        Self {
            file_name: file_name.clone(),
            square_size,
            mode,
        }
    }
}

fn main() {
    let settings = Settings::from(env::args());

    let file = match File::open(&settings.file_name) {
        Ok(f) => f,
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => fail(format!("Error: \"{}\" not found", settings.file_name)),
            io::ErrorKind::PermissionDenied => fail(format!(
                "Error: Permission denied for the file \"{}\"",
                settings.file_name
            )),
            k => fail(format!(
                "Error: Unknown error when trying to open \"{}\": {k:?}",
                settings.file_name
            )),
        },
    };

    let img = ImageReader::new(BufReader::new(file))
        .with_guessed_format()
        .unwrap_or_else(|_| fail("Error: Error while reading the image"));
    if !matches!(img.format(), Some(ImageFormat::Jpeg)) {
        fail("Error: The provided image must be in the JPEG format");
    }

    let img = img.decode().expect("Error: Failed to decode the image");
    let img = match display_size() {
        Ok((w, h)) => {
            let (w, h) = (w as u32, h as u32);
            if img.width() > w || img.height() > h {
                img.resize(w, h, image::imageops::FilterType::CatmullRom)
            } else {
                img
            }
        }
        Err(e) => {
            eprintln!(
                "Warning: Could not obtain the display size ({e:?}) - the image will not be downsized",
            );
            img
        }
    };

    let rgb = img.to_rgb8();
    let pixels: Vec<AtomicU32> = rgb
        .pixels()
        .map(|p| AtomicU32::new((p[0] as u32) << 16 | (p[1] as u32) << 8 | p[2] as u32))
        .collect();
    let pixels = Arc::new(pixels);

    visualize(
        rgb.width(),
        rgb.height(),
        pixels,
        settings.square_size.get(),
    );
}
