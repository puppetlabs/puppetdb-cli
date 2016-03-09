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
    let mut ctx = SslContext::new(SslMethod::Tlsv1_2).unwrap();
    try!(ctx.set_cipher_list("TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256"));
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

#[derive(RustcDecodable, RustcEncodable)]
pub struct Config {
    pub server_urls: Vec<String>,
    pub cacert: Option<String>,
    pub cert: Option<String>,
    pub key: Option<String>
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
    urls.split(",").map(|u| u.to_string()).collect()
}

#[test]
fn parse_server_urls_works() {
    assert_eq!(vec!["http://localhost:8080  ".to_string(),
                    "http://foo.bar.baz:9190".to_string() ],
               parse_server_urls(
                   "http://localhost:8080  ,http://foo.bar.baz:9190".to_string()))
}


#[derive(RustcDecodable, RustcEncodable)]
pub struct CLIConfig {
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
        if !cacert.is_empty() { config.cacert = Some(cacert) };
        if !cert.is_empty() { config.cert = Some(cert) };
        if !key.is_empty() { config.key = Some(key) };
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
                println_stderr!("Error parsing config ------> {:?}: {}", path, e);
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
            cacert: None,
            cert: None,
            key: None
        }
    }
}

pub fn client(config: &Config) -> Client {
    match config {
        &Config{ cacert: Some(ref cacert),
                 cert: Some(ref cert),
                 key: Some(ref key), ..} => Client::with_connector(ssl_connector(Path::new(&cacert),
                                                                                 Path::new(&cert),
                                                                                 Path::new(&key))),
        _ => Client::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;       
    use rustc_serialize::json;
    use std::io::{Write,Error};
    use std::path::{PathBuf};

    extern crate tempdir;
    use self::tempdir::*;

    fn create_temp_path(temp_dir: &TempDir,file_name: &str) -> PathBuf {
        temp_dir.path().join(file_name)
    }

    fn spit_config(file_path: &str, config: &CLIConfig) -> Result<(),Error> {
        let mut f = try!(File::create(file_path));
        try!(f.write_all(json::encode(config).unwrap().as_bytes()));
        Ok(())
    }

    #[test]
    fn load_test_all_fields(){
        let config = CLIConfig { 
            puppetdb: Config {
                server_urls: vec!["http://foo".to_string()],
                cacert: Some("foo".to_string()),
                cert: Some("bar".to_string()),
                key: Some("baz".to_string())
            }
        };

        let temp_dir= TempDir::new_in("target","test-").unwrap();
        let temp_path = create_temp_path(&temp_dir, "testfile.json");
        let path_str = temp_path.as_path().to_str().unwrap();

        spit_config(path_str, &config).unwrap();
        let slurped_config = Config::load(path_str.to_string());

        assert_eq!(config.puppetdb.server_urls[0], slurped_config.server_urls[0]);
        assert_eq!(config.puppetdb.cacert, slurped_config.cacert);
        assert_eq!(config.puppetdb.cert, slurped_config.cert);
        assert_eq!(config.puppetdb.key, slurped_config.key)
    }

    fn spit_string(file_path: &str, contents: &str) -> Result<(),Error> {
        let mut f = try!(File::create(file_path));
        try!(f.write_all(contents.as_bytes()));
        Ok(())
    }

    #[test]
    fn load_test_only_urls(){

        let temp_dir = TempDir::new_in("target","test-").unwrap();
        let temp_path= create_temp_path(&temp_dir, "testfile.json");
        let path_str = temp_path.as_path().to_str().unwrap();

        spit_string(&path_str, "{\"puppetdb\":{\"server_urls\":[\"http://foo\"]}}").unwrap();
        let slurped_config = Config::load(path_str.to_string());

        assert_eq!("http://foo", slurped_config.server_urls[0]);
        assert_eq!(None, slurped_config.cacert);
        assert_eq!(None, slurped_config.cert);
        assert_eq!(None, slurped_config.key);
    }
}
