extern crate rustc_serialize;
extern crate docopt;
extern crate puppetdb;
extern crate beautician;
extern crate hyper;

use std::io::{self, Read, Write};
use std::process;
use docopt::Docopt;

const USAGE: &'static str = "
puppet-db.

Usage:
  puppet-db [options] (--version | --help)
  puppet-db [options] export <path> [--anon=<profile>]
  puppet-db [options] import <path>
  puppet-db [options] status

Options:
  -h --help           Show this screen.
  -v --version        Show version.
  --anon=<profile>    Archive anonymization [default: none].
  -c --config=<path>  Path to CLI config, defaults to $HOME/.puppetlabs/client-tools/puppetdb.conf.
  -u --urls=<urls>    Urls to PuppetDB instances.
  --cacert=<path>     Path to CA certificate for auth.
  --cert=<path>       Path to client certificate for auth.
  --key=<path>        Path to client private key for auth.
";

use puppetdb::client;

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_version: bool,
    flag_anon: String,
    flag_config: String,
    flag_urls: String,
    flag_cacert: String,
    flag_cert: String,
    flag_key: String,
    arg_path: String,
    cmd_import: bool,
    cmd_export: bool,
    cmd_status: bool,
}

use std::fs::File;
use std::env;

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());
    if args.flag_version {
        println!("puppet-db v{}", VERSION.unwrap_or("unknown"));
        return;
    }

    let path: String = if args.flag_config.is_empty() {
        match env::home_dir() {
            Some(mut conf_dir) => {
                conf_dir.push(".puppetlabs");
                conf_dir.push("client-tools");
                conf_dir.push("puppetdb");
                conf_dir.set_extension("conf");
                conf_dir.to_str().unwrap().to_owned()
            },
            None => panic!("$HOME directory is not configured"),
        }
    } else {
        args.flag_config
    };

    let config = client::Config::new(path,
                                     args.flag_urls,
                                     args.flag_cacert,
                                     args.flag_cert,
                                     args.flag_key);
    if args.cmd_export {
        let path = args.arg_path;
        match client::execute_export(config, args.flag_anon) {
            Ok(mut response) => {
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
                    process::exit(1)
                };
                match File::create(path.clone()) {
                    Ok(mut f) => {
                        match io::copy(&mut response, &mut f) {
                            Ok(_) => println!("Wrote archive to {:?}.", path),
                            Err(e) => panic!("Error writing to archive: {}", e)
                        };
                    },
                    Err(x) => panic!("Unable to create archive: {}", x),
                };
            },
            Err(e) => {
                match writeln!(&mut std::io::stderr(), "Failed to connect to PuppetDB: {}", e) {
                    Ok(_) => {},
                    Err(x) => panic!("Unable to write to stderr: {}", x),
                };
                process::exit(1)
            },
        };
    } else if args.cmd_import {
        let path = args.arg_path;
        match client::execute_import(config, path.clone()) {
            Ok(mut response) => {
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
                    process::exit(1)
                };
            },
            Err(e) => {
                match writeln!(&mut std::io::stderr(), "Failed to connect to PuppetDB: {}", e) {
                    Ok(_) => {},
                    Err(x) => panic!("Unable to write to stderr: {}", x),
                };
                process::exit(1)
            },
        }
    } else if args.cmd_status {
        match client::execute_status(config) {
            Ok(mut response) => {
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
                    process::exit(1)
                }

                let stdout = io::stdout();
                let mut handle = stdout.lock();
                beautician::prettify(&mut response, &mut handle).ok().expect("failed to write response");
            },
            Err(e) => {
                match writeln!(&mut std::io::stderr(), "Failed to connect to PuppetDB: {}", e) {
                    Ok(_) => {},
                    Err(x) => panic!("Unable to write to stderr: {}", x),
                };
                process::exit(1)
            },
        };
    }
}
