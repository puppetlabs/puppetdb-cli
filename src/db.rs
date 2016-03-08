extern crate rustc_serialize;
extern crate docopt;
#[macro_use(println_stderr)]
extern crate puppetdb;
extern crate beautician;
extern crate hyper;

use std::io::{self, Write};
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
use puppetdb::admin;
use puppetdb::utils;

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

/// Copies the response body to a file with the given path.
fn copy_response_to_file(res: utils::Result, path: String) {
    match res {
        Ok(mut response) => {
            utils::assert_status_ok(&mut response);
            match File::create(path.clone()) {
                Ok(mut f) => {
                    if let Err(e) = io::copy(&mut response, &mut f) {
                        panic!("Error writing to archive: {}", e);
                    }
                    println!("Wrote archive to {:?}.", path)
                },
                Err(x) => panic!("Unable to create archive: {}", x),
            };
        },
        Err(e) => {
            println_stderr!("Failed to connect to PuppetDB: {}", e);
            process::exit(1)
        },
    };
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());
    if args.flag_version {
        println!("puppet-db v{}", VERSION.unwrap_or("unknown"));
        return;
    }

    let path: String = if args.flag_config.is_empty() {
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
    if args.cmd_export {
        let path = args.arg_path;
        let res = admin::get_export(&config, args.flag_anon);
        copy_response_to_file(res, path);
    } else if args.cmd_import {
        let path = args.arg_path;
        match admin::post_import(&config, path.clone()) {
            Ok(mut response) => utils::assert_status_ok(&mut response),
            Err(e) => {
                println_stderr!("Failed to connect to PuppetDB: {}", e);
                process::exit(1)
            },
        }
    } else if args.cmd_status {
        let resp = admin::get_status(&config);
        utils::prettify_response_to_stdout(resp);
    }
}
