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
  --config=<path>   Path to CLI config, defaults to $HOME/.puppetlabs/client-tools/puppetdb.conf.
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
use std::io::{self, Read};
use std::process;
use std::env;

fn load_config (path: String) -> client::Config {
    let mut f = File::open(&path).ok().expect("Couldn't open config file.");
    let mut s = String::new();
    f.read_to_string(&mut s).ok().expect("Couldn't read from config file.");
    match json::Json::from_str(&s) {
        Ok(parsed_config) => {
            match parsed_config.as_object() {
                Some(raw_config) => match raw_config.get("default_environment") {
                    Some(default_env) => match default_env {
                        &json::Json::String(ref env) => {
                            let encoded = raw_config.get("environments").unwrap().as_object().unwrap()
                                .get(env);
                            match json::decode(&encoded.unwrap().to_string()) {
                                Ok(config) => config,
                                Err(err) => { println!("error parsing config file: {}", err);
                                              println!("Using default PuppetDB location at 'http://127.0.0.1:8080'.");
                                              println!("");
                                              Default::default() },
                            }
                        },
                        _ => { println!("error parsing config file: see README for help");
                               println!("Using default PuppetDB location at 'http://127.0.0.1:8080'.");
                               println!("");
                               Default::default() },
                    },
                    None => { println!("error parsing config file: see README for help");
                              println!("Using default PuppetDB location at 'http://127.0.0.1:8080'.");
                              println!("");
                              Default::default() },
                },
                None => { println!("error parsing config file: see README for help");
                          println!("Using default PuppetDB location at 'http://127.0.0.1:8080'.");
                          println!("");
                          Default::default() },
            }
        }
        Err(err) => { println!("error parsing config file: {}", err);
                      println!("Using default PuppetDB location at 'http://127.0.0.1:8080'.");
                      println!("");
                      Default::default() },
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

        match env::home_dir() {
            Some(mut conf_dir) => {
                conf_dir.push(".puppetlabs");
                conf_dir.push("client-tools");
                conf_dir.push("puppetdb");
                conf_dir.set_extension("conf");
                let path = conf_dir.to_str().unwrap().to_owned();
                match File::open(&path).ok() {
                    Some(_) => load_config(path),
                    None =>  { println!("**** Warning ****");
                               println!("*");
                               println!("*  Can't open config at '~/.puppetlabs/client-tools/puppetdb.conf'.");
                               println!("*  Using default PuppetDB location at 'http://127.0.0.1:8080'.");
                               println!("*");
                               println!("*****************");
                               println!("");
                               Default::default() },
                }
            },
            None => { println!("**** Warning ****");
                      println!("*");
                      println!("*  $HOME directory is not configured.");
                      println!("*  Can't open config at '~/.puppetlabs/client-tools/puppetdb.conf'.");
                      println!("*  Using default PuppetDB location at 'http://127.0.0.1:8080'.");
                      println!("*");
                      println!("*****************");
                      println!("");
                      Default::default() },
        }

    } else {
        load_config(args.flag_config)
    };
    if args.cmd_query {
        let query_str = args.arg_query.expect("Please specify a query for PuppetDB.");
        let option = client::execute_query(config, query_str).ok();
        match option {
            Some(mut response) => {
                let stdout = io::stdout();
                let mut handle = stdout.lock();
                io::copy(&mut response, &mut handle).ok().expect("failed to write response");
            },
            None => { println!("failed to connect to PuppetDB");
                      process::exit(1); },
        };
    } else if args.cmd_export {
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
