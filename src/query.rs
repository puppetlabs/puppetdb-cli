extern crate rustc_serialize;
extern crate docopt;

extern crate serde;
extern crate serde_json;
extern crate serde_transcode;

#[macro_use]
extern crate kitchensink;
extern crate puppetdb;

use serde_json::{Serializer, Deserializer};
use std::io::{self, Read, Write, BufReader, BufWriter};

use puppetdb::client;
use puppetdb::config;

use kitchensink::utils;

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

fn prettify<R: Read, W: Write>(mut r: R, mut w: W) {
    let reader = BufReader::new(&mut r);
    let writer = BufWriter::new(&mut w);

    let mut deserializer = Deserializer::from_reader(reader);
    let mut serializer = Serializer::pretty(writer);
    serde_transcode::transcode(&mut deserializer, &mut serializer)
        .unwrap_or_else(|e| pretty_panic!("Failed to write response: {}", e));
    serializer.into_inner().flush()
        .unwrap_or_else(|e| pretty_panic!("Failed to write response: {}", e));
}

#[test]
fn prettify_handles_utf8() {
    let in_string = &b"[{ \"message\": \"\xE3\x83\x96\xE3\x83\xAC\xE3\x83\x83\xE3\x83\x88\"}]"[..];
    let out_string = "[\n  {\n    \"message\": \"ブレット\"\n  }\n]";
    let mut temp: Vec<u8> = Vec::new();
    prettify(in_string, &mut temp);
    assert_eq!(out_string.to_string(),String::from_utf8(temp).unwrap());
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
        config::default_config_path()
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
    prettify(resp, stdout.lock());
}
