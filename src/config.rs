use std::io::{Read, Write};
use std::process;
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

fn parse_server_urls(urls: String) -> Vec<String> {
    urls.split(",").map(|u| u.trim().to_string()).collect()
}

#[test]
fn parse_server_urls_works() {
    assert_eq!(vec!["http://localhost:8080".to_string(),
                    "http://foo.bar.baz:9190".to_string() ],
               parse_server_urls("   http://localhost:8080  ,   http://foo.bar.baz:9190".to_string()))
}

#[derive(RustcDecodable,Clone)]
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
                key: Option<String>) -> Config {

        if urls.is_some() && cacert.is_some() && cert.is_some() && key.is_some() {
            return Config {
                server_urls: parse_server_urls(urls.unwrap()),
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
        } = match CLIConfig::load(path).puppetdb {
            Some(section) => section,
            None => default_pdb_config_section(),
        };

        // TODO Add tests for Config parsing edge cases
        Config {
            server_urls:
            if let Some(urls) = urls { parse_server_urls(urls) }
            else { cfg_urls.unwrap() },
            cacert: if cacert.is_some() { cacert } else { cfg_cacert },
            cert: if cert.is_some() { cert } else { cfg_cert },
            key: if key.is_some() { key } else { cfg_key },
            token: None,
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
        if !Path::new(&path).exists() {
            return CLIConfig {
                puppetdb: Some(default_pdb_config_section())
            };
        }
        let mut f = match File::open(&path) {
            Ok(d) => d,
            Err(e) => {
                println_stderr!("Error opening config {:?}: {}", path, e);
                process::exit(1)
            }
        };
        let mut s = String::new();
        if let Err(e) = f.read_to_string(&mut s) {
            println_stderr!("Error reading from config {:?}: {}", path, e);
            process::exit(1)
        }
        match json::decode(&s) {
            Ok(d) => d,
            Err(e) => {
                println_stderr!("Error parsing config {:?}: {}", path, e);
                process::exit(1)
            }
        }
    }
}
