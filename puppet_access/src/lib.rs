extern crate libc;

use libc::c_char;
use std::ffi::CStr;
use libc::{free, c_void};

use std::str;

#[link(name = "puppet-access")]
extern {
    fn read_token(input: *const c_char) -> *mut c_char;
    fn get_default_token_file() -> *mut c_char;
}

/// Reads the contents of a token file from the input path.
///
/// Returns `None` if the token does not exist.
pub fn read_token_file(input: String) -> Option<String> {
    unsafe {
        let y_ptr = read_token(input.as_ptr() as *const i8);
        let y = CStr::from_ptr(y_ptr);
        free(y_ptr as *mut c_void);
        str::from_utf8(y.to_bytes())
            .ok()
            .and_then(|s| Some(s.to_owned()) )
            .and_then(|s| if s.is_empty() { None } else { Some(s) })
    }
}

/// Gets the default token file location.
///
/// Returns `None` if the something went wrong.
pub fn default_token_file() -> Option<String> {
    unsafe {
        let x_ptr = get_default_token_file();
        let x = CStr::from_ptr(x_ptr);
        free(x_ptr as *mut c_void);
        str::from_utf8(x.to_bytes())
            .ok()
            .and_then(|s| Some(s.to_owned()) )
            .and_then(|s| if s.is_empty() { None } else { Some(s) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::prelude::*;
    use std::fs::{self,File};

    #[test]
    fn it_works() {
        // Run `cargo test -- --nocapture` to see the output from `println!`
        let x = default_token_file().unwrap();

        let mut f = File::create(&x).unwrap();
        f.write_all(b"fkgjh95 ghdlfjgh").unwrap();

        assert_eq!("fkgjh95 ghdlfjgh", read_token_file(x.clone()).unwrap());
        assert_eq!("", read_token_file("/fake/token".to_string()).unwrap());

        fs::remove_file(x).unwrap();
    }
}
