extern crate libc;

use std::io::{self, Read};
use std::fs::File;

use libc::c_char;
use std::ffi::CStr;

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
pub fn default_token_file() -> String {
    unsafe {
        CStr::from_ptr(get_default_token_file())
            .to_string_lossy()
            .into_owned()
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
        let x = default_token_file();

        let mut f = File::create(&x).unwrap();
        f.write_all(b"fkgjh95 ghdlfjgh").unwrap();

        assert_eq!("fkgjh95 ghdlfjgh", read_token(x.clone()).unwrap());
        assert_eq!("", read_token("/fake/token".to_string()).unwrap());

        fs::remove_file(x).unwrap();
    }
}
