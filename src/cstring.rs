#![deny(missing_docs)]
#![allow(dead_code)]

/// A null-terminated C string.
/// Implemented as a thin wrapper around String.
pub struct CString {
    data: String,
}

impl CString {
    /// Creates an empty CString.
    pub fn new() -> Self {
        CString {
            data: String::new(),
        }
    }

    /// Creates a reference to the underlying String.
    pub fn as_str(self: &Self) -> &str {
        self.data.as_str()
    }
}

/// Creates a CString from a byte slice. Stops at the first null byte.
/// Generic over the slice size.
impl<const N: usize> From<&[u8; N]> for CString {
    fn from(slice: &[u8; N]) -> Self {
        CString {
            data: slice
                .iter()
                .take_while(|b| **b != 0)
                .map(|b| *b as char)
                .collect(),
        }
    }
}

impl From<*const u8> for CString {
    fn from(mut ptr: *const u8) -> CString {
        let mut data = String::new();
        unsafe {
            while !ptr.is_null() {
                data.push(*ptr as char);
                ptr = ptr.offset(1);
            }
        }
        CString { data }
    }
}

impl ToString for CString {
    fn to_string(self: &Self) -> String {
        self.data.clone()
    }
}
