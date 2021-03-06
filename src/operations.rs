use std::io::prelude::*;
use std::fs;
use std::fs::File;
use std::io::SeekFrom;
use std::io;
use std::os::unix::fs::PermissionsExt;

use crate::archive::*;

/// Function that opens a tar file for extracting or just 
/// showing the contents
pub fn extract_files(f: &mut File, action: Action) -> Result<(), io::Error> {
    loop {
        let head = UstarHeader::read_header(f);

        // If filename is empty, consider archive over
        if head.file_name[0] == 0 {
            break;
        }
        
        if head.checksum() != head.compute_checksum() {
            return Err(io::Error::new(io::ErrorKind::Other, "Checksum verification failed"));
        }

        head.display_file_info();

        let size = head.file_size();
        // Tar files are partitioned into 512 byte chunks,
        // padded with zeroes if data doesn't fill them 
        let chunks = (size / 512) + 1;

        if let Action::Extract = action {
            match head.file_type() {
                FileType::Normal => {
                    let mut newfile = File::create(head.file_name())?;
                    for i in 0..chunks {
                        let mut buffer: [u8; 512] = [0; 512];
                        f.read(&mut buffer)?;
                        // The last chunk is padded with zeroes, but they mustn't be written to file
                        if i == chunks - 1 {
                            newfile.write_all(&buffer[0..(size - (chunks - 1) * 512)])?;
                        }
                        else {
                            newfile.write_all(&buffer[..])?;
                        }
                    }
                },
                FileType::Directory => {
                    fs::create_dir_all(head.file_name())?;
                }
                _ => { return Err(io::Error::new(io::ErrorKind::Other, "Filetype not implemented")); }
            }
            // Set the file's or dir's permissions
            fs::set_permissions(head.file_name(), fs::Permissions::from_mode(head.to_numeric_mode()))?;
        }
        else if let Action::Display = action {
            if let FileType::Normal = head.file_type() {
                // Jump the contents chunks if just displaying info
                f.seek(SeekFrom::Current((chunks * 512) as i64))?;
            }
        }

        
        println!();
    }
    Ok(())
}

/// Function for building a tar file from a list of files
pub fn archive_files(f: &mut File, files: Vec<String>) -> Result<(), io::Error> {
    for file_name in files.iter() {
        let mut file = File::open(file_name)?;
        let header = UstarHeader::header_from_file(&file, file_name);
        let size = header.file_size();
        let chunks = (size / 512) + 1;

        f.write_all(&header.serialize_to_array())?;

        if let FileType::Normal = header.file_type() {
            for i in 0..chunks {
                let mut buffer: [u8; 512] = [0; 512];
                let _ = file.read(&mut buffer)?;
                f.write_all(&buffer)?;
            }
        }
        else if let FileType::Directory = header.file_type() {
            let contents = fs::read_dir(file_name)?;
            let mut paths: Vec<String> = Vec::new();

            for path in contents {
                // Ugly, but it seems to be the standard Rust way to read directory contents
                paths.push(path.unwrap().path().display().to_string().clone());
            }
            
            // Recursively call the function for the directory contents
            archive_files(f, paths)?;
        }
    }
    Ok(())
}

#[derive(Debug)]
pub enum Action {
    Extract,
    Display,
    Archive,
    Nop
}