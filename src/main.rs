//! A Rust alternative to `tar`.
#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications
)]
#![feature(const_generics)]
use operations::Action;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use structopt::{clap::ArgGroup, StructOpt};

mod archive;
mod cstring;
mod operations;

#[derive(StructOpt, Debug)]
#[structopt(name = "rtar", group = ArgGroup::with_name("action").required(true))]
struct Arguments {
    #[structopt(short = "v", group = "action")]
    /// Show file contents
    view: bool,

    #[structopt(short = "c", group = "action", requires("files-to-archive"))]
    /// Add files to archive
    compress: bool,

    #[structopt(short = "x", group = "action")]
    /// Extract archive
    extract: bool,

    #[structopt(parse(from_os_str))]
    /// Tar file to process
    archive_file: PathBuf,

    #[structopt(short = "f")]
    /// Files to add to archive
    files_to_archive: Option<Vec<String>>,
}

fn main() {
    let args = Arguments::from_args();
    let action = {
        if args.view {
            Action::Display
        } else if args.extract {
            Action::Extract
        } else {
            Action::Archive
        }
    };

    match action {
        Action::Extract => {
            let mut f = File::open(
                args.archive_file
                    .into_os_string()
                    .into_string()
                    .expect("ERROR bad path"),
            )
            .expect("ERROR opening file");
            let ret = operations::extract_files(&mut f, Action::Extract);

            if let Err(e) = ret {
                eprintln!("ERROR: {}", e);
                std::process::exit(1);
            }
        }
        Action::Display => {
            let mut f = File::open(
                args.archive_file
                    .into_os_string()
                    .into_string()
                    .expect("ERROR bad path"),
            )
            .expect("ERROR opening file");
            let ret = operations::extract_files(&mut f, Action::Display);

            if let Err(e) = ret {
                eprintln!("ERROR: {}", e);
                std::process::exit(1);
            }
        }
        Action::Archive => {
            let mut file = File::create(
                args.archive_file
                    .into_os_string()
                    .into_string()
                    .expect("ERROR bad path"),
            )
            .expect("ERROR creating file");
            let to_archive = args.files_to_archive.expect("ERROR no files to archive");
            let ret = operations::archive_files(&mut file, to_archive);

            if let Err(e) = ret {
                eprintln!("ERROR: {}", e);
                std::process::exit(1);
            }

            let trailer: [u8; 1024] = [0; 1024];
            file.write_all(&trailer)
                .expect("ERROR couldn't write to file");
        }
        Action::Nop => {}
    }
}
