use std::io::{self, Read};
use std::fs::File;
use std::path::PathBuf;

/// Reads the contents of a token file from the input path.
pub fn read_token(path: String) -> io::Result<String> {
    let mut f = try!(File::open(&path));
    let mut s = String::new();
    try!(f.read_to_string(&mut s));
    Ok(s.trim().to_owned())
}

/// Given a `home_dir` (e.g. from `std::env::home_dir()`), returns the default
/// location of the token, `$HOME/.puppetlabs/token`.
pub fn default_token_path(mut home_dir: PathBuf) -> String {
    home_dir.push(".puppetlabs");
    home_dir.push("token");
    home_dir.to_str().unwrap().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::prelude::*;
    use std::fs::{self,File};
    use std::env;

    #[test]
    fn it_works() {
        // Run `cargo test -- --nocapture` to see the output from `println!`
        let conf_dir = env::home_dir().expect("$HOME directory is not configured");
        let x = default_token_path(conf_dir);

        let mut f = File::create(&x).unwrap();
        f.write_all(b"fkgjh95 ghdlfjgh").unwrap();

        assert_eq!("fkgjh95 ghdlfjgh", read_token(x.clone()).unwrap());
        assert_eq!("", read_token("/fake/token".to_string()).unwrap());

        fs::remove_file(x).unwrap();
    }
}
