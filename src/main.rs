mod averaging;
mod multi;
mod picture;
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

use crate::{picture::Picture, visualization::visualize};

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

    let img = img
        .decode()
        .unwrap_or_else(|e| fail(format!("Error: Failed to decode the image ({e:?})")));

    let (window_w, window_h) = match display_size() {
        Ok((w, h)) => {
            let scale = (w as f64 / img.width() as f64)
                .min(h as f64 / img.height() as f64)
                .min(1.0);
            let (w, h) = (
                ((img.width() as f64 * scale) as u32).max(1),
                ((img.height() as f64 * scale) as u32).max(1),
            );
            println!(
                "Info: The image was downsized from {}x{} to {w}x{h} since it did not fit in the display",
                img.width(),
                img.height()
            );
            (w, h)
        }
        Err(e) => {
            eprintln!(
                "Warning: Could not obtain the display size ({e:?}) - the image will not be downsized",
            );
            (img.width(), img.height())
        }
    };

    let rgb = img.to_rgb8();
    let pixels: Vec<AtomicU32> = rgb
        .pixels()
        .map(|p| AtomicU32::new((p[0] as u32) << 16 | (p[1] as u32) << 8 | p[2] as u32))
        .collect();

    let picture = Arc::new(Picture::new(pixels, rgb.width(), rgb.height()));

    let spawn_workers = match &settings.mode {
        ProcessingMode::Single => single::spawn_worker,
        ProcessingMode::Multi => multi::spawn_workers,
    };

    visualize(
        picture,
        settings.square_size.get(),
        window_w,
        window_h,
        spawn_workers,
    );
}
