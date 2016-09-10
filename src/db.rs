extern crate rustc_serialize;
extern crate docopt;

#[macro_use]
extern crate kitchensink;
extern crate puppetdb;

use rustc_serialize::json;
use std::io::{self, Write};
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
  --token=<path>      Path to RBAC token for auth (PE Only).
";

use puppetdb::client;
use puppetdb::config;
use puppetdb::admin;
use kitchensink::utils;

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
    flag_token: Option<String>,
    arg_path: String,
    cmd_import: bool,
    cmd_export: bool,
    cmd_status: bool,
}

use std::fs::File;

/// Copies the response body to a file with the given path.
fn copy_response_to_file(resp: &mut utils::HyperResponse, path: String) {
    utils::assert_status_ok(resp);
    match File::create(path.clone()) {
        Ok(mut f) => {
            io::copy(resp, &mut f).unwrap_or_else(|e| panic!("Error writing to archive: {}", e));
            println!("Wrote archive to {:?}.", path)
        }
        Err(x) => panic!("Unable to create archive: {}", x),
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
        config::default_config_path()
    };

    let config = config::Config::load(path,
                                      args.flag_urls,
                                      args.flag_cacert,
                                      args.flag_cert,
                                      args.flag_key,
                                      args.flag_token);
    let client = client::PdbClient::new(config);

    if args.cmd_export {
        let mut resp = admin::get_export(&client, args.flag_anon)
            .unwrap_or_else(|e| pretty_panic!("Failed to connect to server: {}", e));
        copy_response_to_file(&mut resp, args.arg_path);
    } else if args.cmd_import {
        let mut resp = admin::post_import(&client, args.arg_path)
            .unwrap_or_else(|e| pretty_panic!("Failed to connect to server: {}", e));
        utils::assert_status_ok(&mut resp);
    } else if args.cmd_status {
        println!("{}", json::as_pretty_json(&client.status()));
    }
}
