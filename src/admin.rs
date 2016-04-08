use std::io::Write;
use url::Url;

use multipart::client::Multipart;
use hyper::header::Connection;
use hyper::method::Method;

use super::client::PdbClient;
use super::net::Auth;
use super::utils::HyperResult;

/// POSTs a multipart request to PuppetDB for importing an archive.
pub fn post_import(pdb_client: &PdbClient, path: String) -> HyperResult {
    // Import and export are not prepared to use token auth
    let server_url: String = pdb_client.server_urls[0].clone();
    let url = Url::parse(&(server_url + "/pdb/admin/v1/archive")).unwrap();
    let request = Auth::request(&pdb_client.auth, Method::Post, url);
    let mut multipart = Multipart::from_request(request).unwrap();
    multipart.write_file("archive", &path)
             .unwrap_or_else(|e| pretty_panic!("Error writing archive to request: {}", e));
    multipart.send()
}

pub fn get_export(pdb_client: &PdbClient, anonymization: String) -> HyperResult {
    // Import and export are not prepared to use token auth
    let server_url: String = pdb_client.server_urls[0].clone();
    let body = "anonymization=".to_string() + &anonymization;
    let cli = Auth::client(&pdb_client.auth);

    let req = cli.get(&(server_url + "/pdb/admin/v1/archive"))
        .body(&body)
        .header(Connection::close());
    Auth::auth_header(&pdb_client.auth, req).send()
}
