#![deny(missing_docs)]
use crate::cstring::CString;
use std::fs::File;
use std::io::prelude::*;
use std::os::linux::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::time::SystemTime;

// Constants for header section sizes.
const HEADER: usize = 512;
const FILE_NAME: usize = 100;
const FILE_MODE: usize = 8;
const UID: usize = 8;
const GID: usize = 8;
const FILE_SIZE: usize = 12;
const MODIFIED: usize = 12;
const CHECKSUM: usize = 8;
const FILE_TYPE: usize = 1;
const LINKED_FILE: usize = 100;
const USTAR: usize = 8;
const USERNAME: usize = 32;
const GROUPNAME: usize = 32;
const MAJOR_NUMBER: usize = 8;
const MINOR_NUMBER: usize = 8;
const FILE_PREFIX: usize = 155;
const PADDING: usize = 12;

#[derive(Debug)]
pub struct UstarHeader {
    pub file_name: [u8; FILE_NAME],
    pub file_mode: [u8; FILE_MODE],
    pub uid: [u8; UID],
    pub gid: [u8; GID],
    pub file_size: [u8; FILE_SIZE],
    pub modified: [u8; MODIFIED],
    pub checksum: [u8; CHECKSUM],
    pub file_type: [u8; FILE_TYPE],
    pub linked_file: [u8; LINKED_FILE],
    pub ustar: [u8; USTAR],
    pub username: [u8; USERNAME],
    pub groupname: [u8; GROUPNAME],
    pub major_number: [u8; MAJOR_NUMBER],
    pub minor_number: [u8; MINOR_NUMBER],
    pub file_prefix: [u8; FILE_PREFIX],
    pub padding: [u8; PADDING],
}

#[derive(Debug)]
pub enum FileType {
    Normal,
    Directory,
    HardLink,
    SymLink,
}

impl UstarHeader {
    pub fn new() -> UstarHeader {
        UstarHeader {
            file_name: [0; FILE_NAME],
            file_mode: [0; FILE_MODE],
            uid: [0; UID],
            gid: [0; GID],
            file_size: [0; FILE_SIZE],
            modified: [0; MODIFIED],
            checksum: [b' '; CHECKSUM],
            file_type: [0; FILE_TYPE],
            linked_file: [0; LINKED_FILE],
            ustar: [b'u', b's', b't', b'a', b'r', b' ', b' ', 0u8],
            username: [0; USERNAME],
            groupname: [0; GROUPNAME],
            major_number: [0; MAJOR_NUMBER],
            minor_number: [0; MINOR_NUMBER],
            file_prefix: [0; FILE_PREFIX],
            padding: [0; PADDING],
        }
    }

    /// Returns the file name
    ///
    /// It builds it from the prefix and suffix if needed
    pub fn file_name(&self) -> String {
        // Name is split into 2 if larger than 100 characters
        let filename = CString::from(&self.file_name).to_string();
        let prefix = CString::from(&self.file_prefix).to_string();
        prefix + &filename
    }

    /// Returns the size of the file
    pub fn file_size(&self) -> usize {
        i64::from_str_radix(CString::from(&self.file_size).as_str(), 8)
            .expect("ERROR parsing file size") as usize
    }

    /// Returns the checksum value
    pub fn checksum(&self) -> u32 {
        i64::from_str_radix(CString::from(&self.checksum).as_str(), 8)
            .expect("ERROR parsing file size") as u32
    }

    /// Computes the checksum based on the rest of the header
    pub fn compute_checksum(&self) -> u32 {
        let mut cksum: u32 = 256;

        // Lambda for looping over the field and adding the sum
        let mut checksummer = |arr: &[u8]| {
            for nr in arr {
                cksum += *nr as u32;
            }
        };

        checksummer(&self.file_name);
        checksummer(&self.file_mode);
        checksummer(&self.uid);
        checksummer(&self.gid);
        checksummer(&self.file_size);
        checksummer(&self.modified);
        checksummer(&self.file_type);
        checksummer(&self.linked_file);
        checksummer(&self.ustar);
        checksummer(&self.username);
        checksummer(&self.groupname);
        checksummer(&self.major_number);
        checksummer(&self.minor_number);
        checksummer(&self.file_prefix);
        checksummer(&self.padding);

        cksum
    }

    /// Prints brief info about the file
    pub fn display_file_info(&self) {
        println!(
            "File: {}\nSize: {}\nType: {:?}",
            self.file_name(),
            self.file_size(),
            self.file_type()
        );
    }

    /// Returns the type of the file
    pub fn file_type(&self) -> FileType {
        match i64::from_str_radix(CString::from(&self.file_type).as_str(), 8)
            .expect("ERROR parsing file type")
        {
            0 => FileType::Normal,
            1 => FileType::HardLink,
            2 => FileType::SymLink,
            5 => FileType::Directory,
            _ => panic!("ERROR file type not supported"),
        }
    }

    /// Builds a UstarHeader from a given file
    pub fn from_file(file: &File, name: &String) -> UstarHeader {
        let mut header = UstarHeader::new();

        // Handle name splitting
        if name.len() > 99 {
            let mut prefix = name.clone();
            prefix.truncate(154);
            header.file_prefix[..prefix.len()].copy_from_slice(&prefix.as_bytes());

            if name.len() > 154 {
                let mut suffix = String::from(&name[154..]);
                suffix.truncate(99);
                header.file_name[..suffix.len()].copy_from_slice(&suffix.as_bytes());
            }
        } else {
            header.file_name[..name.len()].copy_from_slice(name.as_bytes());
        }

        let metadata = file.metadata().expect("ERROR getting metadata");

        // File permissions in octal
        header.file_mode[..FILE_MODE - 1]
            .copy_from_slice(format!("{:0>7o}", metadata.permissions().mode()).as_bytes());

        // UID and GID in octal
        header.uid[..UID - 1].copy_from_slice(format!("{:0>7o}", metadata.st_uid()).as_bytes());
        header.gid[..7].copy_from_slice(format!("{:0>7o}", metadata.st_gid()).as_bytes());

        // Modification time is also Unix time in octal
        let modified_time = metadata
            .modified()
            .expect("ERROR getting modified time")
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("ERROR converting time")
            .as_secs();
        header.modified[..MODIFIED - 1]
            .copy_from_slice(format!("{:0>11o}", modified_time).as_bytes());

        if metadata.is_dir() {
            header.file_type = ['5' as u8];
            // Tar uses size of 0 for directories even if on disk they are not 0
            header.file_size = [0; FILE_SIZE]
        } else if metadata.is_file() {
            header.file_type = ['0' as u8];
            header.file_size[..FILE_SIZE - 1]
                .copy_from_slice(format!("{:0>11o}", metadata.len()).as_bytes());
        } else {
            // Store type "a" for unimplemented types.
            // TODO: find a better alternative.
            header.file_type = ['a' as u8];
            header.file_size[..FILE_SIZE - 1]
                .copy_from_slice(format!("{:0>11o}", metadata.len()).as_bytes());
        }

        // Checksum
        let checksum = header.compute_checksum();
        header.checksum[..CHECKSUM - 1].copy_from_slice(format!("{:0>6o} ", checksum).as_bytes());

        header
    }

    /// Reads a file header from a tar file
    pub fn read_header(f: &mut File) -> UstarHeader {
        let mut buffer = [0; HEADER];
        f.read_exact(&mut buffer[..]).expect("ERROR reading header");

        let mut header = UstarHeader::new();
        header.file_name.copy_from_slice(&buffer[0..100]);
        header.file_mode.copy_from_slice(&buffer[100..108]);
        header.uid.copy_from_slice(&buffer[108..116]);
        header.gid.copy_from_slice(&buffer[116..124]);
        header.file_size.copy_from_slice(&buffer[124..136]);
        header.modified.copy_from_slice(&buffer[136..148]);
        header.checksum.copy_from_slice(&buffer[148..156]);
        header.file_type.copy_from_slice(&buffer[156..157]);
        header.linked_file.copy_from_slice(&buffer[157..257]);
        header.ustar.copy_from_slice(&buffer[257..265]);
        header.username.copy_from_slice(&buffer[265..297]);
        header.groupname.copy_from_slice(&buffer[297..329]);
        header.major_number.copy_from_slice(&buffer[329..337]);
        header.minor_number.copy_from_slice(&buffer[337..345]);
        header.file_prefix.copy_from_slice(&buffer[345..500]);
        header
    }

    /// Copies a header into a 512 byte array
    pub fn serialize_to_array(&self) -> [u8; HEADER] {
        let mut buffer = [0; HEADER];

        buffer[0..100].copy_from_slice(&self.file_name);
        buffer[100..108].copy_from_slice(&self.file_mode);
        buffer[108..116].copy_from_slice(&self.uid);
        buffer[116..124].copy_from_slice(&self.gid);
        buffer[124..136].copy_from_slice(&self.file_size);
        buffer[136..148].copy_from_slice(&self.modified);
        buffer[148..156].copy_from_slice(&self.checksum);
        buffer[156..157].copy_from_slice(&self.file_type);
        buffer[157..257].copy_from_slice(&self.linked_file);
        buffer[257..265].copy_from_slice(&self.ustar);
        buffer[265..297].copy_from_slice(&self.username);
        buffer[297..329].copy_from_slice(&self.groupname);
        buffer[329..337].copy_from_slice(&self.major_number);
        buffer[337..345].copy_from_slice(&self.minor_number);
        buffer[345..500].copy_from_slice(&self.file_prefix);

        buffer
    }

    /// Converts the mode(permissions) of the file to a numeric value
    pub fn to_numeric_mode(&self) -> u32 {
        i64::from_str_radix(CString::from(&self.file_mode).as_str(), 8)
            .expect("ERROR parsing file mode") as u32
    }
}
