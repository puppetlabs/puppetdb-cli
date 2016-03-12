extern crate hyper;

use multipart::client::Multipart;
use url::Url;

use hyper::net::Streaming;
use hyper::client::request::Request;
use hyper::header::Connection;

use super::client::PdbClient;
use super::net::Auth;
use super::utils::Result;

/// POSTs a multipart request to PuppetDB for importing an archive.
pub fn post_import(pdb_client: &PdbClient, path: String) -> Result {
    // Import and export are not prepared to use token auth
    let server_url: String = pdb_client.server_urls[0].clone();
    let url = Url::parse(&(server_url + "/pdb/admin/v1/archive")).unwrap();
    let mut multipart: Multipart<Request<Streaming>> = Auth::multipart(&pdb_client.auth, url);
    multipart.write_file("archive", &path);
    multipart.send()
}

pub fn get_export(pdb_client: &PdbClient, anonymization: String) -> Result {
    // Import and export are not prepared to use token auth
    let server_url: String = pdb_client.server_urls[0].clone();
    Auth::client(&pdb_client.auth)
        .get(&(server_url + "/pdb/admin/v1/archive"))
        .body(&("anonymization=".to_string() + &anonymization))
        .header(Connection::close())
        .send()
}

pub fn get_status(pdb_client: &PdbClient) -> Result {
    let server_url: String = pdb_client.server_urls[0].clone();
    let cli = Auth::client(&pdb_client.auth);
    let mut req = cli
        .post(&(server_url + "/status/v1/services"))
        .header(Connection::close());
    if let Some(auth) = Auth::auth_header(&pdb_client.auth) {
        req = req.header(auth)
    };
    req.send()
}
