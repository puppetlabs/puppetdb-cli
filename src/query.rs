extern crate rustc_serialize;
extern crate docopt;
extern crate puppetdb;
extern crate beautician;
extern crate hyper;

use std::env;

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
        let conf_dir = env::home_dir()
            .expect("$HOME directory is not configured");
        config::default_config_path(conf_dir)
    };

    let config = config::Config::load(path,
                                      args.flag_urls,
                                      args.flag_cacert,
                                      args.flag_cert,
                                      args.flag_key);
    let client = client::PdbClient::new(config);
    let query_str = args.arg_query.unwrap();
    let resp = client::post_query(&client, query_str);
    utils::prettify_response_to_stdout(resp);
}
