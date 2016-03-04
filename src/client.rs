use std::io::{self, Read, Write};
use std::process;
use hyper::Client;
use hyper::header::{Connection,ContentType};
use hyper::method::Method;
use hyper::client::request::Request;
use hyper::client::response::Response;
use hyper::error::Error;

use rustc_serialize::json::{self,ToJson};

#[derive(RustcEncodable)]
pub struct PdbRequest {
    query: json::Json,
}
use multipart::client::Multipart;
use url::Url;

use std::path::Path;
use openssl::ssl::{SslContext,SslMethod};
use openssl::ssl::error::SslError;
use openssl::x509::X509FileType;
use std::sync::Arc;
use hyper::net::Openssl;
pub fn ssl_context<C>(cacert: C, cert: C, key: C) -> Result<Openssl, SslError>
    where C: AsRef<Path> {
    let mut ctx = SslContext::new(SslMethod::Sslv23).unwrap();
    try!(ctx.set_cipher_list("DEFAULT"));
    try!(ctx.set_CA_file(cacert.as_ref()));
    try!(ctx.set_certificate_file(cert.as_ref(), X509FileType::PEM));
    try!(ctx.set_private_key_file(key.as_ref(), X509FileType::PEM));
    Ok(Openssl { context: Arc::new(ctx) })
}

use hyper::net::HttpsConnector;
pub fn ssl_connector<C>(cacert: C, cert: C, key: C) -> HttpsConnector<Openssl>
    where C: AsRef<Path> {
    let ctx = ssl_context(cacert, cert, key).ok().expect("error opening certificate files");
    HttpsConnector::new(ctx)
}

#[derive(RustcDecodable)]
pub struct Config {
    pub server_urls: Vec<String>,
    pub cacert: String,
    pub cert: String,
    pub key: String,
}

fn parse_server_urls(urls: String) -> Vec<String> {
    urls.split(",").map(|u| u.to_string() ).collect()
}

impl Config {
    pub fn new(path: String,
               urls: String,
               cacert: String,
               cert: String,
               key: String) -> Config {
       let mut config: Config =
            if !urls.is_empty() && !cacert.is_empty() && !cert.is_empty() && !key.is_empty() {
                Default::default()
            } else {
                match File::open(&path).ok() {
                    Some(_) => load_config(path),
                    None => panic!("Can't open config at {:?}", path),
                }
            };
        if !urls.is_empty() {
            config.server_urls = parse_server_urls(urls.clone())
        };
        if !cacert.is_empty() { config.cacert = cacert };
        if !cert.is_empty() { config.cert = cert };
        if !key.is_empty() { config.key = key };
        config
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            server_urls: vec!["http://127.0.0.1:8080".to_string()],
            cacert: "".to_string(),
            cert: "".to_string(),
            key: "".to_string(),
        }
    }
}

pub fn client(config: Config) -> Client {
    if !config.cacert.is_empty() {
        let conn = ssl_connector(Path::new(&config.cacert),
                                 Path::new(&config.cert),
                                 Path::new(&config.key));
        Client::with_connector(conn)
    } else {
        Client::new()
    }
}

use hyper::net::Streaming;
pub fn multipart(config: Config, url: Url) -> Multipart<Request<Streaming>> {
    let request =
        if !config.cacert.is_empty() {
            let conn = ssl_connector(Path::new(&config.cacert),
                                     Path::new(&config.cert),
                                     Path::new(&config.key));
            Request::with_connector(Method::Post, url, &conn).unwrap()
        } else {
            Request::new(Method::Post, url).unwrap()
        };
    Multipart::from_request(request).unwrap()
}

pub fn execute_import(config: Config, path: String) -> Result<Response,Error> {
    let server_url: String = config.server_urls[0].clone();
    let url = Url::parse(&(server_url + "/pdb/admin/v1/archive")).unwrap();
    let mut multipart = multipart(config, url);
    multipart.write_file("archive", &path);
    multipart.send()
}

pub fn execute_export(config: Config, anonymization: String) -> Result<Response,Error> {
    let server_url: String = config.server_urls[0].clone();
    client(config)
        .get(&(server_url + "/pdb/admin/v1/archive"))
        .body(&("anonymization=".to_string() + &anonymization))
        .header(Connection::close())
        .send()
}

pub fn execute_query(config: Config, query_str: String) -> Result<Response,Error> {
    let server_url: String = config.server_urls[0].clone();
    let query = if query_str.trim().starts_with("[") {
        json::Json::from_str(&query_str).unwrap()
    } else {
        query_str.to_json()
    };
    let pdb_query = PdbRequest{query: query};
    let pdb_query_str = json::encode(&pdb_query).unwrap().to_string();

    client(config)
        .post(&(server_url + "/pdb/query/v4"))
        .body(&pdb_query_str)
        .header(ContentType::json())
        .header(Connection::close())
        .send()
}

pub fn execute_status(config: Config) -> Result<Response,Error> {
    let server_url: String = config.server_urls[0].clone();

    client(config)
        .get(&(server_url + "/status/v1/services"))
        .header(Connection::close())
        .send()
}

#[derive(RustcDecodable,Default)]
struct CLIConfig {
    puppetdb: Config,
}

use std::fs::File;

pub fn load_config(path: String) -> Config {
    let mut f = File::open(&path).ok().expect("Couldn't open config file.");
    let mut s = String::new();
    f.read_to_string(&mut s).ok().expect("Couldn't read from config file.");
    let cli_config: CLIConfig = match json::decode(&s) {
        Ok(d) => d,
        Err(e) => {
            match writeln!(&mut io::stderr(), "Error parsing config {:?}: {}", path, e) {
                Ok(_) => {},
                Err(x) => panic!("Unable to write to stderr: {}", x),
            };
            process::exit(1)
        }
    };
    let mut config: Config = cli_config.puppetdb;
    config.server_urls = if config.server_urls.len() > 0 {
        config.server_urls
    } else {
        vec!["http://127.0.0.1:8080".to_string()]
    };
    config
}
