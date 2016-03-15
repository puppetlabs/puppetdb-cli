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
use puppetdb::config;
use puppetdb::admin;
use puppetdb::utils;

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_version: bool,
    flag_anon: String,
    flag_config: Option<String>,
    flag_urls: Option<String>,
    flag_cacert: Option<String>,
    flag_cert: Option<String>,
    flag_key: Option<String>,
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
                }
                Err(x) => panic!("Unable to create archive: {}", x),
            };
        }
        Err(e) => {
            println_stderr!("Failed to connect to PuppetDB: {}", e);
            process::exit(1)
        }
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

    let path = if let Some(cfg_path) = args.flag_config {
        cfg_path
    } else {
        let conf_dir = env::home_dir().expect("$HOME directory is not configured");
        config::default_config_path(conf_dir)
    };

    let config = config::Config::load(path,
                                      args.flag_urls,
                                      args.flag_cacert,
                                      args.flag_cert,
                                      args.flag_key);
    let client = client::PdbClient::new(config);
    if args.cmd_export {
        let path = args.arg_path;
        let res = admin::get_export(&client, args.flag_anon);
        copy_response_to_file(res, path);
    } else if args.cmd_import {
        let path = args.arg_path;
        match admin::post_import(&client, path.clone()) {
            Ok(mut response) => utils::assert_status_ok(&mut response),
            Err(e) => {
                println_stderr!("Failed to connect to PuppetDB: {}", e);
                process::exit(1)
            }
        }
    } else if args.cmd_status {
        admin::get_status(&client);
    }
}
