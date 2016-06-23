extern crate rustc_serialize;
extern crate docopt;
extern crate beautician;
extern crate hyper;

#[macro_use]
extern crate puppetdb;

use std::env;
use std::io::{self, Write};

use puppetdb::client;
use puppetdb::utils;
use puppetdb::config;

use docopt::Docopt;

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
  --token=<path>      Path to RBAC token for auth (PE Only).
";

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_version: bool,
    flag_config: Option<String>,
    flag_urls: Option<String>,
    flag_cacert: Option<String>,
    flag_cert: Option<String>,
    flag_key: Option<String>,
    flag_token: Option<String>,
    arg_query: Option<String>,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());
    if args.flag_version {
        println!("puppet-query v{}", VERSION.unwrap_or("unknown"));
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
                                      args.flag_key,
                                      args.flag_token);

    let mut resp = client::PdbClient::new(config)
        .query(args.arg_query.unwrap())
        .unwrap_or_else(|e| pretty_panic!("Failed to connect to server: {}", e));

    utils::assert_status_ok(&mut resp);

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    beautician::prettify(&mut resp, &mut handle)
        .ok()
        .expect("failed to write response");
}
