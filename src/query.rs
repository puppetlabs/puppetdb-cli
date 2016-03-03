extern crate rustc_serialize;
extern crate docopt;

use docopt::Docopt;

const USAGE: &'static str = "
puppet-query.

Usage:
  puppet-query [options] (--version | --help)
  puppet-query [options] <query>

Options:
  -h --help         Show this screen.
  -v --version      Show version.
  --config=<path>   Path to CLI config, defaults to $HOME/.puppetlabs/client-tools/puppetdb.conf.
";

extern crate puppetdb;
use puppetdb::client;

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_version: bool,
    flag_config: String,
    arg_query: Option<String>,
}

extern crate beautician;
use std::fs::File;
use std::io::{self, Read, Write};
use std::env;

extern crate hyper;
fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());
    if args.flag_version {
        println!("puppet-query v{}", VERSION.unwrap_or("unknown"));
        return;
    }

    let config: client::Config = if args.flag_config.is_empty() {
        match env::home_dir() {
            Some(mut conf_dir) => {
                conf_dir.push(".puppetlabs");
                conf_dir.push("client-tools");
                conf_dir.push("puppetdb");
                conf_dir.set_extension("conf");
                let path = conf_dir.to_str().unwrap().to_owned();
                match File::open(&path).ok() {
                    Some(_) => client::load_config(path),
                    None =>  Default::default(),
                }
            },
            None => Default::default(),
        }
    } else {
        let path = args.flag_config;
        match File::open(&path).ok() {
            Some(_) => client::load_config(path),
            None => panic!("Can't open config at {:?}", path),
        }
    };
    let query_str = args.arg_query.unwrap();
    let option = client::execute_query(config, query_str).ok();
    match option {
        Some(mut response) => {
            let status = response.status;
            if status != hyper::Ok {
                let mut temp = String::new();
                match response.read_to_string(&mut temp) {
                    Ok(_) => {},
                    Err(x) => panic!("Unable to read response from server: {}", x),
                };
                match writeln!(&mut std::io::stderr(), "Error response from server: {}", temp) {
                    Ok(_) => {},
                    Err(x) => panic!("Unable to write to stderr: {}", x),
                };
                std::process::exit(1)
            }

            let stdout = io::stdout();
            let mut handle = stdout.lock();
            beautician::prettify(&mut response, &mut handle).ok().expect("failed to write response");
        },
        None => panic!("failed to connect to PuppetDB"),
    };
}
