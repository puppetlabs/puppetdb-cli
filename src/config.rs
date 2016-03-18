use std::io::{Read, Write};
use std::fs::File;
use std::path::{Path, PathBuf};

use rustc_serialize::json;

/// Given a `home_dir` (e.g. from `std::env::home_dir()`), returns the default
/// location of the client configuration file,
/// `$HOME/.puppetlabs/client-tools/puppetdb.conf`.
pub fn default_config_path(mut home_dir: PathBuf) -> String {
    home_dir.push(".puppetlabs");
    home_dir.push("client-tools");
    home_dir.push("puppetdb");
    home_dir.set_extension("conf");
    home_dir.to_str().unwrap().to_owned()
}

fn split_server_urls(urls: String) -> Vec<String> {
    urls.split(",").map(|u| u.trim().to_string()).collect()
}

#[test]
fn split_server_urls_works() {
    assert_eq!(vec!["http://localhost:8080".to_string(), "http://foo.bar.baz:9190".to_string()],
               split_server_urls("   http://localhost:8080  ,   http://foo.bar.baz:9190"
                                     .to_string()))
}

#[derive(RustcDecodable,Clone,Debug)]
pub struct Config {
    pub server_urls: Vec<String>,
    pub cacert: Option<String>,
    pub cert: Option<String>,
    pub key: Option<String>,
    pub token: Option<String>,
}

impl Config {
    pub fn load(path: String,
                urls: Option<String>,
                cacert: Option<String>,
                cert: Option<String>,
                key: Option<String>,
                token: Option<String>)
                -> Config {

        // TODO Don't parse config if urls aren't HTTP. This is trivial but it
        // would be best to merge the other auth validation code when
        // constructing the client with this.
        if urls.is_some() && cacert.is_some() && cert.is_some() && key.is_some() {
            return Config {
                server_urls: split_server_urls(urls.unwrap()),
                cacert: cacert,
                cert: cert,
                key: key,
                token: None,
            };
        }

        let PdbConfigSection {
            server_urls: cfg_urls,
            cacert: cfg_cacert,
            cert: cfg_cert,
            key: cfg_key,
        } = if !Path::new(&path).exists() {
            default_pdb_config_section()
        } else {
            match CLIConfig::load(path).puppetdb {
                Some(section) => section,
                None => default_pdb_config_section(),
            }
        };

        // TODO Add tests for Config parsing edge cases
        Config {
            server_urls: urls.and_then(|s| {
                                 if s.is_empty() {
                                     None
                                 } else {
                                     Some(s)
                                 }
                             })
                             .and_then(|s| Some(split_server_urls(s)))
                             .or(cfg_urls)
                             .unwrap_or(default_server_urls()),
            cacert: cacert.or(cfg_cacert),
            cert: cert.or(cfg_cert),
            key: key.or(cfg_key),
            token: token,
        }
    }
}

#[derive(RustcDecodable,Debug)]
struct PdbConfigSection {
    server_urls: Option<Vec<String>>,
    cacert: Option<String>,
    cert: Option<String>,
    key: Option<String>,
}

fn default_server_urls() -> Vec<String> {
    vec!["http://127.0.0.1:8080".to_string()]
}

fn default_pdb_config_section() -> PdbConfigSection {
    PdbConfigSection {
        server_urls: Some(vec!["http://127.0.0.1:8080".to_string()]),
        cacert: None,
        cert: None,
        key: None,
    }
}

#[derive(RustcDecodable,Debug)]
struct CLIConfig {
    puppetdb: Option<PdbConfigSection>,
}

impl CLIConfig {
    fn load(path: String) -> CLIConfig {
        let mut f = File::open(&path).unwrap_or_else(|e| {
            pretty_panic!("Error opening config {:?}: {}", path, e)
        });
        let mut s = String::new();
        if let Err(e) = f.read_to_string(&mut s) {
            pretty_panic!("Error reading from config {:?}: {}", path, e)
        }
        json::decode(&s).unwrap_or_else(|e| pretty_panic!("Error parsing config {:?}: {}", path, e))
    }
}
