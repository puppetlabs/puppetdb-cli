extern crate hyper;
extern crate beautician;
use std::io::{self, Read, Write};
use std::process;
use std::result;

/// Like `println!` but for stderr.
#[macro_export]
macro_rules! println_stderr(
    ($($arg:tt)*) => (
        match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
    )
);

/// Exits with an error if the response did not have status 200.
pub fn assert_status_ok(response: &mut hyper::client::response::Response) {
    if response.status != hyper::Ok {
        let mut temp = String::new();
        if let Err(x) = response.read_to_string(&mut temp) {
            panic!("Unable to read response from server: {}", x);
        }
        println_stderr!("Error response from server: {}", temp);
        process::exit(1)
    }
}

/// Type alias for the result of a hyper HTTP request.
pub type Result = result::Result<hyper::client::response::Response,
                                 hyper::error::Error>;

/// Pretty prints the response body to stdout.
pub fn prettify_response_to_stdout(res: Result) {
    match res {
        Ok(mut response) => {
            assert_status_ok(&mut response);
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            beautician::prettify(&mut response, &mut handle).ok().expect("failed to write response");
        },
        Err(e) => {
            println_stderr!("Failed to connect to PuppetDB: {}", e);
            process::exit(1)
        },
    };
}

