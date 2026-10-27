use std::{env, fs::File, num::NonZeroU32, process};

#[derive(Debug)]
pub enum ProcessingMode {
    Single,
    Multi,
}

pub fn main() {
    let args: Vec<String> = env::args().collect();
    let Some([_, file_name, square_size, mode]) = args.as_array() else {
        eprintln!(
            "Usage: {} <filename> <square size> <processing mode>",
            args.first().unwrap()
        );
        process::exit(1);
    };

    let Ok(square_size) = square_size.parse::<NonZeroU32>() else {
        eprintln!("Square size must be a positive integer");
        process::exit(1);
    };

    let mode = match mode.to_uppercase().as_str() {
        "S" => ProcessingMode::Single,
        "M" => ProcessingMode::Multi,
        _ => {
            eprintln!(
                "The processing mode must be either 'S' (single threaded) or 'M' (multi threaded)"
            );
            process::exit(1);
        }
    };

    let Ok(file) = File::open(file_name) else {
        eprintln!("\"{file_name}\" does not exist or cannot be opened");
        process::exit(1);
    };

    println!("Args: {file:?}\n{square_size}\n{mode:?}");
}
