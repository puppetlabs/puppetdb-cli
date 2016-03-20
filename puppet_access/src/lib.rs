extern crate libc;

use std::io::{self, Read};
use std::fs::File;

use libc::c_char;
use std::ffi::CStr;
use libc::{free, c_void};

use std::str;

#[link(name = "puppet-access")]
extern {
    fn get_default_token_file() -> *mut c_char;
}

/// Reads the contents of a token file from the input path.
pub fn read_token(path: String) -> io::Result<String> {
    let mut f = try!(File::open(&path));
    let mut s = String::new();
    try!(f.read_to_string(&mut s));
    Ok(s)
}

/// Gets the default token file location.
///
/// Returns `None` if the something went wrong.
pub fn default_token_file() -> Option<String> {
    unsafe {
        let x_ptr = get_default_token_file();
        let x = CStr::from_ptr(x_ptr);
        let rstr = str::from_utf8(x.to_bytes())
            .ok()
            .and_then(|s| Some(s.to_owned()) )
            .and_then(|s| if s.is_empty() { None } else { Some(s) });

        free(x_ptr as *mut c_void);

        rstr
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
