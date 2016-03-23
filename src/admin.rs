extern crate hyper;

use std::path::Path;
use multipart::client::Multipart;
use url::Url;

use hyper::net::Streaming;
use hyper::method::Method;
use hyper::client::request::Request;
use hyper::header::Connection;

use super::client::{self, Config};
use super::utils::Result;

/// Construct a multipart request.
fn multipart(config: &Config, url: Url) -> Multipart<Request<Streaming>> {
    let request =
        if !config.cacert.is_empty() {
            let conn = client::ssl_connector(Path::new(&config.cacert),
                                             Path::new(&config.cert),
                                             Path::new(&config.key));
            Request::with_connector(Method::Post, url, &conn).unwrap()
        } else {
            Request::new(Method::Post, url).unwrap()
        };
    Multipart::from_request(request).unwrap()
}

/// POSTs a multipart request to PuppetDB for importing an archive.
pub fn post_import(config: &Config, path: String) -> Result {
    let server_url: String = config.server_urls[0].clone();
    let url = Url::parse(&(server_url + "/pdb/admin/v1/archive")).unwrap();
    let mut multipart = multipart(config, url);
    multipart.write_file("archive", &path).unwrap();
    multipart.send()
}

pub fn get_export(config: &Config, anonymization: String) -> Result {
    let server_url: String = config.server_urls[0].clone();
    client::client(config)
        .get(&(server_url + "/pdb/admin/v1/archive"))
        .body(&("anonymization=".to_string() + &anonymization))
        .header(Connection::close())
        .send()
}

pub fn get_status(config: &Config) -> Result {
    let server_url: String = config.server_urls[0].clone();
    client::client(config)
        .get(&(server_url + "/status/v1/services"))
        .header(Connection::close())
        .send()
}
