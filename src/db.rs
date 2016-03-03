extern crate rustc_serialize;
extern crate docopt;

use docopt::Docopt;

const USAGE: &'static str = "
puppet-db.

Usage:
  puppet-db [options] (--version | --help)
  puppet-db [options] export <path> [--anon=<profile>]
  puppet-db [options] import <path>

Options:
  -h --help         Show this screen.
  -v --version      Show version.
  --anon=<profile>  Archive anonymization [default: none].
  --config=<path>   Path to CLI config, defaults to $HOME/.puppetlabs/client-tools/puppetdb.conf.
";

extern crate puppetdb;
use puppetdb::client;

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_version: bool,
    flag_config: String,
    flag_anon: String,
    arg_path: Option<String>,
    cmd_import: bool,
    cmd_export: bool,
}

use rustc_serialize::json;
use std::fs::File;
use std::io::{self, Read};
use std::process;
use std::env;

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());
    if args.flag_version {
        println!("puppet-db v{}", VERSION.unwrap_or("unknown"));
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
    if args.cmd_export {
        let path = args.arg_path.expect("Please specify the archive file to export PuppetDB to.");
        let option = client::execute_export(config, args.flag_anon).ok();
        match option {
            Some(mut response) => {
                let mut f = File::create(path.clone()).ok().expect("failed to create file");
                io::copy(&mut response, &mut f).ok().expect("failed to write response");
                println!("Wrote archive to {}.", path);
            },
            None => { println!("failed to connect to PuppetDB");
                      process::exit(1); },
        };
    } else if args.cmd_import {
        let path = args.arg_path.expect("Please specify the archive file to import to PuppetDB.");
        let option = client::execute_import(config, path.clone()).ok();
        match option {
            Some(mut response) => {
                let mut buffer = String::new();
                response.read_to_string(&mut buffer).unwrap();
                let body_option = json::Json::from_str(&buffer).ok();
                match body_option {
                    Some(body) => {
                        let ok = body.as_object().unwrap().get("ok").unwrap().to_string();
                        if ok == "true" {
                            println!("Import triggered for archive {}", path);
                        } else {
                            println!("error triggering import: check PuppetDB logs for more details");
                            process::exit(1);
                        }
                    },
                    None => { println!("error triggering export: {}", buffer);
                              process::exit(1); }
                };
            },
            None => { println!("failed to connect to PuppetDB");
                      process::exit(1); },
        }
    }
}
