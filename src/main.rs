mod visualization;

use std::{
    env,
    fmt::Display,
    fs::File,
    io::{self, BufReader},
    num::NonZeroU32,
    process,
};

use image::{GenericImageView, ImageFormat, ImageReader};

use crate::visualization::visualize;

#[derive(Debug)]
enum ProcessingMode {
    Single,
    Multi,
}

fn fail(msg: impl Display) -> ! {
    eprintln!("{msg}");
    process::exit(1);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let Some([_, file_name, square_size, mode]) = args.as_array() else {
        let name = args.first().map_or("program", String::as_str);
        fail(format!(
            "Usage: {name} <filename> <square size> <processing mode>",
        ));
    };

    let square_size = square_size
        .parse::<NonZeroU32>()
        .unwrap_or_else(|_| fail("Error: Square size must be a positive integer"));

    let mode = match mode.to_ascii_uppercase().as_str() {
        "S" => ProcessingMode::Single,
        "M" => ProcessingMode::Multi,
        _ => fail(
            "Error: The processing mode must be either 'S' (single threaded) or 'M' (multi threaded)",
        ),
    };

    let file = match File::open(file_name) {
        Ok(f) => f,
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => fail(format!("Error: \"{file_name}\" not found")),
            io::ErrorKind::PermissionDenied => fail(format!(
                "Error: Permission denied for the file \"{file_name}\""
            )),
            _ => fail(format!(
                "Error: Unknown error when trying to open \"{file_name}\""
            )),
        },
    };

    let img = ImageReader::new(BufReader::new(file))
        .with_guessed_format()
        .unwrap_or_else(|_| fail("Error: Error while reading the image"));
    if !matches!(img.format(), Some(ImageFormat::Jpeg)) {
        fail("Error: The provided image must be in the JPEG format");
    }

    let img = img.decode().unwrap();
    let rgb = img.to_rgb8();
    let pixels: Vec<u32> = rgb
        .pixels()
        .map(|p| (p[0] as u32) << 16 | (p[1] as u32) << 8 | p[2] as u32)
        .collect();

    visualize(rgb.width(), rgb.height(), pixels);
}
