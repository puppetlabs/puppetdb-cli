extern crate beautician;
#[macro_use]
extern crate hyper;
extern crate openssl;
extern crate openssl_verify;
extern crate url;
extern crate multipart;
extern crate rustc_serialize;

#[cfg(windows)]
extern crate winapi;
#[cfg(windows)]
extern crate winreg;
#[cfg(windows)]
extern crate shell32;
#[cfg(windows)]
extern crate ole32;
#[cfg(windows)]
extern crate kernel32;
#[cfg(windows)]
extern crate advapi32;
#[cfg(windows)]
extern crate userenv;

#[cfg(feature = "puppet-access")]
extern crate puppet_access;

#[macro_use]pub mod utils;
pub mod net;
pub mod config;
pub mod client;
pub mod admin;

#[cfg(windows)]
pub mod windows {
    use winapi::*;
    use std::io;
    use std::path::PathBuf;
    use std::ptr;
    use std::slice;
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use shell32;
    use ole32;

    #[allow(non_upper_case_globals)]
    pub const FOLDERID_ProgramData: GUID = GUID {
        Data1: 0x62AB5D82,
        Data2: 0xFDC1,
        Data3: 0x4DC3,
        Data4: [0xA9, 0xDD, 0x07, 0x0D, 0x1D, 0x49, 0x5D, 0x97],
    };

    pub fn get_special_folder(id: &shtypes::KNOWNFOLDERID) -> io::Result<PathBuf> {
        let mut path = ptr::null_mut();
        let result;

        unsafe {
            let code = shell32::SHGetKnownFolderPath(id, 0, ptr::null_mut(), &mut path);
            if code == 0 {
                let mut length = 0usize;
                while *path.offset(length as isize) != 0 {
                    length += 1;
                }
                let slice = slice::from_raw_parts(path, length);
                result = Ok(OsString::from_wide(slice).into());
            } else {
                result = Err(io::Error::from_raw_os_error(code));
            }
            ole32::CoTaskMemFree(path as *mut _);
        }
        result
    }
}
