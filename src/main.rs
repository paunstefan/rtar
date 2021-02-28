use std::fs::File;
use std::io::prelude::*;
use std::env;

mod archive;
mod operations;

use operations::Action;

fn main() {

    // Arguments are organized in a special structure for better accessing
    let args: Arguments = {
        let os_args: Vec<String> = env::args().skip(1).collect();
        match Arguments::parse(os_args) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("ERROR: {}", e);
                eprintln!("Usage: rtar -[options] [files]");
                std::process::exit(1);
            }
        }
    };
    
    match args.op {
        Action::Extract => {
            let mut f = File::open(args.archive_file).expect("ERROR opening file");
            let ret = operations::extract_files(&mut f, Action::Extract);

            if let Err(e) = ret {
                eprintln!("ERROR: {}", e);
                std::process::exit(1);
            }
        },
        Action::Display => {
            let mut f = File::open(args.archive_file).expect("ERROR opening file");
            let ret = operations::extract_files(&mut f, Action::Display);

            if let Err(e) = ret {
                eprintln!("ERROR: {}", e);
                std::process::exit(1);
            }
        },
        Action::Archive => {
            let mut file = File::create(args.archive_file).expect("ERROR creating file");
            let ret = operations::archive_files(&mut file, args.to_archive);

            if let Err(e) = ret {
                eprintln!("ERROR: {}", e);
                std::process::exit(1);
            }

            let trailer: [u8; 1024] = [0; 1024];
            file.write_all(&trailer).expect("ERROR couldn't write to file");
         },
        Action::Nop => {}
    }

}

#[derive(Debug)]
struct Arguments {
    op: Action,
    archive_file: String,
    to_archive: Vec<String>,
}

impl Arguments {
    pub fn parse(os_args: Vec<String>) -> Result<Arguments, &'static str> {
        if os_args.len() < 2 {
            return Err("Too few arguments");
        }

        let options: Vec<char> = os_args[0].chars().collect();

        if options.len() < 2 || options[0] != '-' {
            return Err("Options not provided");
        }

        let mut op: Action = Action::Nop;
        let mut archive_file = String::new();
        let mut to_archive: Vec<String> = Vec::new();

        for ch in &options[1..] {
            match *ch {
                'c' => {
                    if let Action::Nop = op {
                        op = Action::Archive;
                        archive_file = os_args[1].clone();
                        
                        for f in os_args.iter().skip(2) {
                            to_archive.push(f.clone());
                        }
                        if to_archive.len() < 1 {
                            return Err("Too few arguments");
                        }
                    }
                    else {
                        return Err("Options incompatible");
                    }
                },
                'x' => {
                    if os_args.len() > 2 {
                        return Err("Too many arguments");
                    }
                    op = Action::Extract;
                    archive_file = os_args[1].clone();
                },
                'v' => {
                    if os_args.len() > 2 {
                        return Err("Too many arguments");
                    }
                    op = Action::Display;
                    archive_file = os_args[1].clone();
                },
                _ => { return Err("Option not recognized"); }
            }
        }

        Ok(Arguments{
            op,
            archive_file,
            to_archive
        })

    }
}