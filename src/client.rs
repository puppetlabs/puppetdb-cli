extern crate hyper;
use std::io::{self, Read, Write};
use std::process;
use std::fs::File;
use std::path::{Path, PathBuf};

use rustc_serialize::json::{self, ToJson};

use openssl::ssl::{SslContext, SslMethod};
use openssl::ssl::error::SslError;
use openssl::x509::X509FileType;

use std::result;
use std::sync::Arc;

use hyper::net::{Openssl, HttpsConnector};
use hyper::Client;
use hyper::header::{Connection, ContentType};

use super::utils::Result;

pub fn ssl_context<C>(cacert: C, cert: C, key: C) -> result::Result<Openssl, SslError>
    where C: AsRef<Path> {
    let mut ctx = SslContext::new(SslMethod::Sslv23).unwrap();
    try!(ctx.set_cipher_list("DEFAULT"));
    try!(ctx.set_CA_file(cacert.as_ref()));
    try!(ctx.set_certificate_file(cert.as_ref(), X509FileType::PEM));
    try!(ctx.set_private_key_file(key.as_ref(), X509FileType::PEM));
    Ok(Openssl { context: Arc::new(ctx) })
}

pub fn ssl_connector<C>(cacert: C, cert: C, key: C) -> HttpsConnector<Openssl>
    where C: AsRef<Path> {
    let ctx = match ssl_context(cacert, cert, key) {
        Ok(ctx) => ctx,
        Err(e) => {
            println_stderr!("Error opening certificate files: {}", e);
            process::exit(1)
        }
    };
    HttpsConnector::new(ctx)
}

#[derive(RustcDecodable)]
pub struct Config {
    pub server_urls: Vec<String>,
    pub cacert: String,
    pub cert: String,
    pub key: String,
}

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

fn default_server_urls() -> Vec<String> {
    vec!["http://127.0.0.1:8080".to_string()]
}

fn parse_server_urls(urls: String) -> Vec<String> {
    urls.split(",").map(|u| u.to_string() ).collect()
}

#[test]
fn parse_server_urls_works() {
    assert_eq!(vec!["http://localhost:8080  ".to_string(),
                    "http://foo.bar.baz:9190".to_string() ],
               parse_server_urls(
                   "http://localhost:8080  ,http://foo.bar.baz:9190".to_string()
               ))
}


#[derive(RustcDecodable,Default)]
struct CLIConfig {
    puppetdb: Config,
}

#[derive(RustcEncodable)]
pub struct PdbRequest {
    query: json::Json,
}

/// Struct to hold PuppetDB client configuration.
impl Config {
    /// Construct new client configuration, intended to be used with command
    /// flags where `path` is the path to the config to load if any of the other
    /// settings are empty.
    pub fn new(path: String,
               urls: String,
               cacert: String,
               cert: String,
               key: String) -> Config {
       let mut config: Config =
            // Do not bother loading the config if the user supplied all the
            // config via flags
            if !urls.is_empty()
            && !cacert.is_empty() && !cert.is_empty() && !key.is_empty() {
                Default::default()
            } else {
                Config::load(path)
            };
        if !urls.is_empty() {
            config.server_urls = parse_server_urls(urls.clone())
        };
        if !cacert.is_empty() { config.cacert = cacert };
        if !cert.is_empty() { config.cert = cert };
        if !key.is_empty() { config.key = key };
        config
    }

    pub fn load(path: String) -> Config {
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
        let cli_config: CLIConfig = match json::decode(&s) {
            Ok(d) => d,
            Err(e) => {
                println_stderr!("Error parsing config {:?}: {}", path, e);
                process::exit(1)
            }
        };
        let mut config: Config = cli_config.puppetdb;
        config.server_urls = if config.server_urls.len() > 0 {
            config.server_urls
        } else {
            default_server_urls()
        };
        config
    }

    /// POSTs `query_str` (either AST or PQL) to configured PuppetDBs.
    pub fn query(&self, query_str: String) -> Result {
        let cli: Client = client(self);
        for server_url in self.server_urls.clone() {
            let query = if query_str.trim().starts_with("[") {
                json::Json::from_str(&query_str).unwrap()
            } else {
                query_str.to_json()
            };
            let pdb_query = PdbRequest{query: query};
            let pdb_query_str = json::encode(&pdb_query).unwrap().to_string();

            let res = cli
                .post(&(server_url + "/pdb/query/v4"))
                .body(&pdb_query_str)
                .header(ContentType::json())
                .header(Connection::close())
                .send();
            if res.is_ok() {
                return res;
            }
        };
        // TODO Collect errors from each server and return them
        let io_error = io::Error::new(
            io::ErrorKind::ConnectionRefused, "connection refused"
        );
        return Err(hyper::error::Error::from(io_error));
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            server_urls: default_server_urls(),
            cacert: String::new(),
            cert: String::new(),
            key: String::new(),
        }
    }
}

pub fn client(config: &Config) -> Client {
    if !config.cacert.is_empty() {
        let conn = ssl_connector(Path::new(&config.cacert),
                                 Path::new(&config.cert),
                                 Path::new(&config.key));
        Client::with_connector(conn)
    } else {
        Client::new()
    }
}
