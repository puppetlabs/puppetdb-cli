use hyper::Client;
use hyper::header::{Connection,ContentType};
use hyper::method::Method;
use hyper::client::request::Request;
use hyper::client::response::Response;
use hyper::error::Error;

use rustc_serialize::json;

#[derive(RustcEncodable)]
pub struct PdbQuery {
    query: json::Json,
}
use multipart::client::Multipart;
use url::Url;
use std::fs::File;
use tar::Archive;
use flate2::read::GzDecoder;

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

#[derive(RustcDecodable, RustcEncodable, Default)]
pub struct Config {
    pub server_urls: Vec<String>,
    pub cacert: String,
    pub cert: String,
    pub key: String,
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

use std::io::Read;

fn get_command_versions(path: &str) -> Option<String> {
    let archive = File::open(&path).ok().expect("couldn't open archive file");
    let gzip = GzDecoder::new(archive).unwrap();
    let mut tar = Archive::new(gzip);
    let mut metadata = String::new();
    for file in tar.files_mut().unwrap() {
        // Make sure there wasn't an I/O error
        let mut file = file.unwrap();

        // Inspect metadata about the file
        if file.header().path().unwrap()
            .into_owned().file_name().unwrap() == "export-metadata.json" {

                file.read_to_string(&mut metadata).unwrap();
                break;
            };
    }
    if metadata.is_empty() {
        None
    } else {
        Some(json::Json::from_str(&metadata).unwrap()
             .as_object().unwrap()
             .get("command_versions").unwrap()
             .to_string())
    }
}

const DEFAULT_COMMAND_VERSIONS: &'static str = "{\"replace_catalog\":7,\"store_report\":6,\"replace_facts\":4}";

pub fn execute_import(config: Config, path: String) -> Result<Response,Error> {
    let server_url: String = if config.server_urls.len() > 0 {
        config.server_urls[0].clone()
    } else {
        "http://127.0.0.1:8080".to_owned()
    };
    let url = Url::parse(&(server_url + "/pdb/admin/v1/archive")).unwrap();
    let mut multipart = multipart(config, url);
    multipart.write_text("command_versions",
                         get_command_versions(&path).unwrap_or(DEFAULT_COMMAND_VERSIONS.to_owned()));
    multipart.write_file("archive", &path);
    multipart.send()
}

pub fn execute_export(config: Config, anonymization: String) -> Result<Response,Error> {
    let server_url: String = if config.server_urls.len() > 0 {
        config.server_urls[0].clone()
    } else {
        "http://127.0.0.1:8080".to_owned()
    };
    client(config)
        .get(&(server_url + "/pdb/admin/v1/archive"))
        .body(&("anonymization=".to_string() + &anonymization))
        .header(Connection::close())
        .send()
}

pub fn execute_query(config: Config, query_str: String) -> Result<Response,Error> {
    let server_url: String = if config.server_urls.len() > 0 {
        config.server_urls[0].clone()
    } else {
        "http://127.0.0.1:8080".to_owned()
    };
    let query = json::Json::from_str(&query_str).unwrap();
    let pdb_query = PdbQuery{query: query};
    let pdb_query_str = json::encode(&pdb_query).unwrap().to_string();

    client(config)
        .post(&(server_url + "/pdb/query/v4"))
        .body(&pdb_query_str)
        .header(ContentType::json())
        .header(Connection::close())
        .send()
}
