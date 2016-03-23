use std::io::Write;
use url::Url;

use hyper::header::Connection;

use super::client::PdbClient;
use super::net::Auth;
use super::utils::HyperResult;

/// POSTs a multipart request to PuppetDB for importing an archive.
pub fn post_import(pdb_client: &PdbClient, path: String) -> HyperResult {
    // Import and export are not prepared to use token auth
    let server_url: String = pdb_client.server_urls[0].clone();
    let url = Url::parse(&(server_url + "/pdb/admin/v1/archive")).unwrap();
    let mut multipart = Auth::multipart(&pdb_client.auth, url);
    multipart.write_file("archive", &path)
             .unwrap_or_else(|e| pretty_panic!("Error writing archive to request: {}", e));
    multipart.send()
}

pub fn get_export(pdb_client: &PdbClient, anonymization: String) -> HyperResult {
    // Import and export are not prepared to use token auth
    let server_url: String = pdb_client.server_urls[0].clone();
    Auth::client(&pdb_client.auth)
        .get(&(server_url + "/pdb/admin/v1/archive"))
        .body(&("anonymization=".to_string() + &anonymization))
        .header(Connection::close())
        .send()
}
