extern crate rustc_serialize;
extern crate docopt;

use docopt::Docopt;

const USAGE: &'static str = "
PuppetDB CLI.

Usage:
  puppet-pdb [options] (--version | --help)
  puppet-pdb [options] query <query>
  puppet-pdb [options] export <path> [--anon=<profile>]
  puppet-pdb [options] import <path>

Options:
  -h --help         Show this screen.
  -v --version      Show version.
  --anon=<profile>  Archive anonymization [default: none].
  --config=<path>   Path to CLI config.
";

extern crate pdb;
use pdb::client;

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_version: bool,
    flag_config: String,
    flag_anon: String,
    arg_query: Option<String>,
    arg_path: Option<String>,
    cmd_query: bool,
    cmd_import: bool,
    cmd_export: bool,
}

use rustc_serialize::json;
use std::fs::File;
use std::io::Read;
fn load_config (path: String) -> client::Config {
    let mut f = File::open(&path).ok().expect("Couldn't open config file.");
    let mut s = String::new();
    f.read_to_string(&mut s).ok().expect("Couldn't read from config file.");
    let parsed_config = json::Json::from_str(&s).unwrap();
    let raw_config = parsed_config.as_object().unwrap();
    match raw_config.get("default_environment").unwrap() {
        &json::Json::String(ref env) => {
            let encoded = raw_config.get("environments").unwrap().as_object().unwrap()
                .get(env);
            json::decode(&encoded.unwrap().to_string()).unwrap()
        },
        _ => Default::default(),
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());
    if args.flag_version {
        println!("PuppetDB CLI v{}", VERSION.unwrap_or("unknown"));
        return;
    }

    let config: client::Config = if args.flag_config.is_empty() {
        Default::default()
    } else {
        load_config(args.flag_config)
    };
    if args.cmd_query {
        let query_str = args.arg_query.expect("Please specify a query for PuppetDB.");
        client::execute_query(config, query_str)
    } else if args.cmd_export {
        let path = args.arg_path.expect("Please specify the archive file to export PuppetDB to.");
        client::execute_export(config, path, args.flag_anon)
    } else if args.cmd_import {
        let path = args.arg_path.expect("Please specify the archive file to import to PuppetDB.");
        client::execute_import(config, path)
    }
}
