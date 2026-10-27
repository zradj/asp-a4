use std::{env, fmt::Display, fs::File, io, num::NonZeroU32, process};

#[derive(Debug)]
pub enum ProcessingMode {
    Single,
    Multi,
}

fn fail(msg: impl Display) -> ! {
    eprintln!("{msg}");
    process::exit(1);
}

pub fn main() {
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

    println!("Args: {file:?}\n{square_size}\n{mode:?}");
}
