use std::{env, fmt::Display, fs::File, num::NonZeroU32, process};

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
        .unwrap_or_else(|_| fail("Square size must be a positive integer"));

    let mode = match mode.to_ascii_uppercase().as_str() {
        "S" => ProcessingMode::Single,
        "M" => ProcessingMode::Multi,
        _ => {
            fail("The processing mode must be either 'S' (single threaded) or 'M' (multi threaded)")
        }
    };

    let file = File::open(file_name).unwrap_or_else(|_| {
        fail(format!(
            "\"{file_name}\" does not exist or cannot be opened"
        ))
    });

    println!("Args: {file:?}\n{square_size}\n{mode:?}");
}
