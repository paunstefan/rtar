use std::fs::File;
use std::io::prelude::*;
use std::env;

mod archive;
mod operations;

use operations::Action;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() < 2 {
        eprintln!("Usage: rtar -[options] [files]");
        std::process::exit(1);
    }

    let options: Vec<char> = args[0].chars().collect();

    if options.len() < 2 || options[0] != '-' {
        eprintln!("Usage: rtar -[options] [files]");
        std::process::exit(1);
    }

    let mut action: Action = Action::Nop;

    for op in &options[1..] {
        match *op {
            'c' => action = Action::Archive,
            'x' => action = Action::Extract,
            'v' => action = Action::Display,
            _ => { eprintln!("ERROR option not recognized"); std::process::exit(1); }
        }
    }

    match action {
        Action::Extract => {
            let mut f = File::open(&args[1]).expect("ERROR opening file");
            operations::extract_files(&mut f, Action::Extract);
        },
        Action::Display => {
            let mut f = File::open(&args[1]).expect("ERROR opening file");
            operations::extract_files(&mut f, Action::Display);
        },
        Action::Archive => {
            let mut file = File::create(&args[1]).expect("ERROR creating file");
            let files: Vec<&String> = args.iter().skip(2).collect();
            operations::archive_files(&mut file, files);

            let trailer: [u8; 1024] = [0; 1024];
            file.write_all(&trailer).expect("ERROR couldn't write to file");
         },
        _ => {}
    }

}
