use std::fs::File;
use std::io::prelude::*;
use std::os::unix::fs::PermissionsExt;
use std::os::linux::fs::MetadataExt;
use std::time::SystemTime;


#[derive(Debug)]
pub struct UstarHeader {
    pub file_name: [u8; 100],
    pub file_mode: [u8; 8],
    pub uid: [u8; 8],
    pub gid: [u8; 8],
    pub file_size: [u8; 12],
    pub modified: [u8; 12],
    pub checksum: [u8; 8],
    pub file_type: [u8; 1],
    pub linked_file: [u8; 100],
    pub ustar: [u8; 8],
    pub username: [u8; 32],
    pub groupname: [u8; 32],
    pub major_number: [u8; 8],
    pub minor_number: [u8; 8],
    pub file_prefix: [u8; 155],
    pub padding: [u8; 12]
}

#[derive(Debug)]
pub enum FileType {
    Normal,
    Directory,
    HardLink,
    SymLink,
}

impl UstarHeader {
    pub fn file_name(&self) -> String {
        // Name is split into 2 if larger than 100 characters
        let filename = ascii_array_to_string(&self.file_name);
        let mut prefix = ascii_array_to_string(&self.file_prefix);

        prefix.push_str(&filename);

        prefix
    }

    pub fn file_size(&self) -> usize {
        let number = ascii_array_to_string(&self.file_size);
        i64::from_str_radix(&number, 8).expect("ERROR parsing file size") as usize
    }

    pub fn checksum(&self) -> u32 {
        i64::from_str_radix(&ascii_array_to_string(&self.checksum), 8).expect("ERROR parsing file size") as u32
    }

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


    pub fn display_file_info(&self) {
        println!("File: {}\nSize: {}\nType: {:?}", 
            self.file_name(), self.file_size(), self.file_type());
    }

    pub fn file_type(&self) -> FileType {
        let number = ascii_array_to_string(&self.file_type);
        match i64::from_str_radix(&number, 8).expect("ERROR parsing file size") {
            0 => FileType::Normal,
            1 => FileType::HardLink,
            2 => FileType::SymLink,
            5 => FileType::Directory,
            _ => panic!("ERROR file type not supported")
        }
    }
    
    pub fn header_from_file(file: &File, name: &String) -> UstarHeader {
        let mut file_name: [u8; 100] = [0; 100];
        let mut file_mode: [u8; 8] = [0; 8];
        let mut uid: [u8; 8] = [0; 8];
        let mut gid: [u8; 8] = [0; 8];
        let mut file_size: [u8; 12] = [0; 12];
        let mut modified: [u8; 12] = [0; 12];
        let mut checksum: [u8; 8] = [b' '; 8];
        let mut file_type: [u8; 1] = [0; 1];
        let mut linked_file: [u8; 100] = [0; 100];
        let mut username: [u8; 32] = [0; 32];
        let mut groupname: [u8; 32] = [0; 32];
        let mut major_number: [u8; 8] = [0; 8];
        let mut minor_number: [u8; 8] = [0; 8];
        let mut file_prefix: [u8; 155] = [0; 155];

        // TODO: files with name bigger than 100
        file_name.copy_from_slice(&string_to_ascii_vec_padded(name, 100));
        
        let metadata = file.metadata().expect("ERROR getting metadata");

        // File permissions in octal
        file_mode.copy_from_slice(&string_to_ascii_vec_padded(&format!("{:0>7o}", metadata.permissions().mode()), 8));

        // UID and GID in octal
        uid.copy_from_slice(&string_to_ascii_vec_padded(&format!("{:0>7o}", metadata.st_uid()), 8));
        gid.copy_from_slice(&string_to_ascii_vec_padded(&format!("{:0>7o}", metadata.st_gid()), 8));

        // Modification time is also Unix time in octal
        let modified_time = metadata.modified().expect("ERROR getting modified time")
                            .duration_since(SystemTime::UNIX_EPOCH).expect("ERROR converting time")
                            .as_secs();
        modified.copy_from_slice(&string_to_ascii_vec_padded(&format!("{:0>11o}", modified_time), 12));
        
        if metadata.is_dir() {
            file_type.copy_from_slice(&string_to_ascii_vec_padded(&String::from("5"), 1));
            // Tar uses size of 0 for directories even if on disk they are not 0
            file_size.copy_from_slice(&string_to_ascii_vec_padded(&format!("{:0>11o}", 0), 12));
        }
        else if metadata.is_file() {
            file_type.copy_from_slice(&string_to_ascii_vec_padded(&String::from("0"), 1));
            file_size.copy_from_slice(&string_to_ascii_vec_padded(&format!("{:0>11o}", metadata.len()), 12));
        }
        else {
            // Temporary I store type "a" for unimplemented types
            file_type.copy_from_slice(&string_to_ascii_vec_padded(&String::from("a"), 1));
            file_size.copy_from_slice(&string_to_ascii_vec_padded(&format!("{:0>11o}", metadata.len()), 12));
        }


        // TODO: rest of fields from libc

        let mut ret = UstarHeader {
            file_name,
            file_mode,
            uid,
            gid,
            file_size,
            modified,
            checksum,
            file_type,
            linked_file,
            ustar: [b'u', b's', b't', b'a', b'r', b' ', b' ', 0u8],
            username,
            groupname,
            major_number,
            minor_number,
            file_prefix,
            padding: [0; 12]
        };

        // Checksum
        checksum.copy_from_slice(&string_to_ascii_vec_padded(&format!("{:0>6o}", ret.compute_checksum()), 8));
        checksum[7] = b' ';
        ret.checksum = checksum;

        ret
    }
    

    pub fn read_header(f: &mut File) -> UstarHeader {
        let mut buffer: [u8; 512] = [0; 512];
    
        f.read(&mut buffer[..]).expect("ERROR reading header");
    
        let mut file_name: [u8; 100] = [0; 100];
        let mut file_mode: [u8; 8] = [0; 8];
        let mut uid: [u8; 8] = [0; 8];
        let mut gid: [u8; 8] = [0; 8];
        let mut file_size: [u8; 12] = [0; 12];
        let mut modified: [u8; 12] = [0; 12];
        let mut checksum: [u8; 8] = [0; 8];
        let mut file_type: [u8; 1] = [0; 1];
        let mut linked_file: [u8; 100] = [0; 100];
        let mut ustar: [u8; 8] = [0; 8];
        let mut username: [u8; 32] = [0; 32];
        let mut groupname: [u8; 32] = [0; 32];
        let mut major_number: [u8; 8] = [0; 8];
        let mut minor_number: [u8; 8] = [0; 8];
        let mut file_prefix: [u8; 155] = [0; 155];
    
        file_name.copy_from_slice(&buffer[0..100]);
        file_mode.copy_from_slice(&buffer[100..108]);
        uid.copy_from_slice(&buffer[108..116]);
        gid.copy_from_slice(&buffer[116..124]);
        file_size.copy_from_slice(&buffer[124..136]);
        modified.copy_from_slice(&buffer[136..148]);
        checksum.copy_from_slice(&buffer[148..156]);
        file_type.copy_from_slice(&buffer[156..157]);
        linked_file.copy_from_slice(&buffer[157..257]);
        ustar.copy_from_slice(&buffer[257..265]);
        username.copy_from_slice(&buffer[265..297]);
        groupname.copy_from_slice(&buffer[297..329]);
        major_number.copy_from_slice(&buffer[329..337]);
        minor_number.copy_from_slice(&buffer[337..345]);
        file_prefix.copy_from_slice(&buffer[345..500]);
    
        UstarHeader {
            file_name,
            file_mode,
            uid,
            gid,
            file_size,
            modified,
            checksum,
            file_type,
            linked_file,
            ustar,
            username,
            groupname,
            major_number,
            minor_number,
            file_prefix,
            padding: [0; 12]
        }
    }

    pub fn serialize_to_array(&self) -> [u8; 512] {
        let mut buffer: [u8; 512] = [0; 512];

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

    pub fn to_numeric_mode(&self) -> u32 {
        i64::from_str_radix(&ascii_array_to_string(&self.file_mode), 8).expect("ERROR parsing file mode") as u32
    }
}

pub fn ascii_array_to_string(arr: &[u8]) -> String {
    let mut ret = String::new();
    for b in arr.iter() {
        if *b == 0 {
            break;
        }
        ret.push(*b as char);
    }

    ret
}


pub fn string_to_ascii_vec_padded(string: &String, size: u32) -> Vec<u8> {
    let mut ret: Vec<u8> = vec![0; size as usize];
    let bytes = string.as_bytes();

    for i in 0..bytes.len() {
        ret[i] = bytes[i];
    }

    ret
}