extern crate rustc_serialize;
extern crate docopt;
extern crate puppetdb;
extern crate beautician;
extern crate hyper;

use std::env;

use puppetdb::client;
use puppetdb::utils;

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
    let resp = config.query(query_str);
    utils::pretty_print_response(resp);
}
