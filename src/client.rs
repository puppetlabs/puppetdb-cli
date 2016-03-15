use std::io::{self, Write};

use rustc_serialize::json::{self, ToJson};

use hyper::error;
use hyper::header::{Connection, ContentType};

use url::Url;

use super::config::Config;
use super::utils::HyperResult;
use super::net::Auth;

pub struct PdbClient {
    pub server_urls: Vec<String>,
    pub auth: Auth,
}

impl PdbClient {
    pub fn new(config: Config) -> PdbClient {
        let is_ssl = config.server_urls
                           .clone()
                           .into_iter()
                           .map(|url: String| {
                               match Url::parse(&url) {
                                   Ok(u) => u.scheme,
                                   Err(e) => pretty_panic!("Error parsing url {:?}: {}", url, e),
                               }
                           })
                           .find(|s| s == "https");

        let result = if is_ssl.is_some() {
            PdbClient::with_auth(config)
        } else {
            PdbClient::without_auth(config)
        };

        match result {
            Ok(pdb_client) => pdb_client,
            Err(e) => pretty_panic!("Error: {}", e),
        }
    }

    pub fn without_auth(config: Config) -> Result<PdbClient, io::Error> {
        Ok(PdbClient {
            server_urls: config.server_urls,
            auth: Auth::NoAuth,
        })
    }

    pub fn with_auth(config: Config) -> Result<PdbClient, io::Error> {
        if config.cacert.is_none() {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                                      "ssl requires 'cacert' to be set"));
        } else {
            if config.cert.is_some() && config.key.is_some() {
                Ok(PdbClient {
                    server_urls: config.server_urls,
                    auth: Auth::CertAuth {
                        cacert: config.cacert.unwrap(),
                        cert: config.cert.unwrap(),
                        key: config.key.unwrap(),
                    },
                })
            } else if config.token.is_some() {
                Ok(PdbClient {
                    server_urls: config.server_urls,
                    auth: Auth::TokenAuth {
                        cacert: config.cacert.unwrap(),
                        token: config.token.unwrap(),
                    },
                })
            } else {
                Err(io::Error::new(io::ErrorKind::InvalidData,
                                   "ssl requires 'cert'/'key' or 'token' to be set"))
            }
        }
    }
}

#[test]
/// Check that `PdbClient::with_auth(Config)` validates the config properly
fn with_auth_works() {

    let no_auth = Config {
        server_urls: vec!["foo".to_string()],
        cacert: None,
        cert: None,
        key: None,
        token: None,
    };
    assert!(PdbClient::with_auth(no_auth).is_err());

    let missing_cacert_token_auth = Config {
        server_urls: vec!["foo".to_string()],
        cacert: None,
        cert: None,
        key: None,
        token: Some("bar".to_string()),
    };
    assert!(PdbClient::with_auth(missing_cacert_token_auth).is_err());

    let missing_cacert_cert_auth = Config {
        server_urls: vec!["foo".to_string()],
        cacert: None,
        cert: Some("bar".to_string()),
        key: Some("bar".to_string()),
        token: None,
    };
    assert!(PdbClient::with_auth(missing_cacert_cert_auth).is_err());

    let missing_cert_cert_auth = Config {
        server_urls: vec!["foo".to_string()],
        cacert: Some("bar".to_string()),
        cert: None,
        key: Some("bar".to_string()),
        token: None,
    };
    assert!(PdbClient::with_auth(missing_cert_cert_auth).is_err());

    let missing_key_cert_auth = Config {
        server_urls: vec!["foo".to_string()],
        cacert: Some("bar".to_string()),
        cert: Some("bar".to_string()),
        key: None,
        token: None,
    };
    assert!(PdbClient::with_auth(missing_key_cert_auth).is_err());

    // CertAuth takes priority over TokenAuth
    let all_auth = Config {
        server_urls: vec!["foo".to_string()],
        cacert: Some("bar".to_string()),
        cert: Some("bar".to_string()),
        key: Some("bar".to_string()),
        token: Some("bar".to_string()),
    };
    assert!(PdbClient::with_auth(all_auth.clone()).ok().is_some());
    assert!(match PdbClient::with_auth(all_auth).unwrap().auth {
        Auth::CertAuth{..} => true,
        _ => false,
    });

    let token_auth = Config {
        server_urls: vec!["foo".to_string()],
        cacert: Some("bar".to_string()),
        cert: None,
        key: None,
        token: Some("bar".to_string()),
    };
    assert!(PdbClient::with_auth(token_auth.clone()).ok().is_some());
    assert!(match PdbClient::with_auth(token_auth).unwrap().auth {
        Auth::TokenAuth{..} => true,
        _ => false,
    });
}

#[derive(RustcEncodable)]
struct PdbQueryRequest {
    query: json::Json,
}

impl PdbQueryRequest {
    fn to_string(&self) -> String {
        json::encode(self).unwrap().to_string()
    }
}

fn query_to_json(query: String) -> json::Json {
    if query.trim().starts_with("[") {
        json::Json::from_str(&query).unwrap()
    } else {
        query.to_json()
    }
}

#[test]
fn query_to_json_works() {
    assert_eq!("\"nodes{ certname ~ \\\".*\\\" }\"",
               query_to_json("nodes{ certname ~ \".*\" }".to_string()).to_string());
    assert_eq!("[\"from\",\"nodes\",[\"~\",\"certname\",\".*\"]]",
               query_to_json("   [\"from\", \"nodes\",[\"~\", \"certname\", \".*\"]]".to_string())
                   .to_string());
}


/// POSTs `query_str` (either AST or PQL) to configured PuppetDBs.
pub fn post_query(pdb_client: &PdbClient, query_str: String) -> HyperResult {

    let cli = Auth::client(&pdb_client.auth);

    let req_body = PdbQueryRequest { query: query_to_json(query_str) }.to_string();

    for server_url in pdb_client.server_urls.clone() {
        let mut req = cli.post(&(server_url + "/pdb/query/v4"))
                         .body(&req_body)
                         .header(ContentType::json())
                         .header(Connection::close());
        if let Some(auth) = Auth::auth_header(&pdb_client.auth) {
            req = req.header(auth)
        };
        let res = req.send();
        if res.is_ok() {
            return res;
        }
    }
    // TODO Collect errors from each server and return them
    let io_error = io::Error::new(io::ErrorKind::ConnectionRefused, "connection refused");
    return Err(error::Error::from(io_error));
}
