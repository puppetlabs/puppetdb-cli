extern crate rustc_serialize;
extern crate docopt;
extern crate puppetdb;
extern crate beautician;
extern crate hyper;

use std::io::{self, Read, Write};
use std::process;
use std::env;

use puppetdb::client;

use docopt::Docopt;

macro_rules! println_stderr(
    ($($arg:tt)*) => (
        match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
    )
);

const USAGE: &'static str = "
puppet-query.

Usage:
  puppet-query [options] (--version | --help)
  puppet-query [options] <query>

Options:
  -h --help           Show this screen.
  -v --version        Show version.
  -c --config=<path>  Path to CLI config, defaults to $HOME/.puppetlabs/client-tools/puppetdb.conf.
  -u --urls=<urls>    Urls to PuppetDB instances.
  --cacert=<path>     Path to CA certificate for auth.
  --cert=<path>       Path to client certificate for auth.
  --key=<path>        Path to client private key for auth.
";

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_version: bool,
    flag_config: String,
    flag_urls: String,
    flag_cacert: String,
    flag_cert: String,
    flag_key: String,
    arg_query: Option<String>,
}

fn pretty_print_response(mut response: hyper::client::response::Response) {
    if response.status != hyper::Ok {
        let mut temp = String::new();
        if let Err(x) = response.read_to_string(&mut temp) {
            panic!("Unable to read response from server: {}", x);
        }
        println_stderr!("Error response from server: {}", temp);
        process::exit(1)
    }

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    beautician::prettify(&mut response, &mut handle).ok().expect("failed to write response");
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());
    if args.flag_version {
        println!("puppet-query v{}", VERSION.unwrap_or("unknown"));
        return;
    }

    let path = if args.flag_config.is_empty() {
        let conf_dir = env::home_dir().expect("$HOME directory is not configured");
        client::default_config_path(conf_dir)
    } else {
        args.flag_config
    };

    let config = client::Config::new(path,
                                     args.flag_urls,
                                     args.flag_cacert,
                                     args.flag_cert,
                                     args.flag_key);
    let query_str = args.arg_query.unwrap();
    match config.query(query_str) {
        Ok(response) => pretty_print_response(response),
        Err(e) => {
            println_stderr!("Failed to connect to PuppetDB: {}", e);
            process::exit(1)
        },
    };
}
