use std::io::{self, Write, Read};
use rustc_serialize::json::{self, ToJson};
use std::collections::BTreeMap;

use hyper;
use hyper::error;
use hyper::header::{Connection, ContentType};

use url::Url;

use super::config::Config;
use super::utils::HyperResult;
use super::net::Auth;

#[cfg(feature = "puppet-access")]
use puppet_access;
#[cfg(feature = "puppet-access")]
use std::env;

/// PuppetDB client struct.
pub struct PdbClient {
    /// List of PuppetDB servers.
    pub server_urls: Vec<String>,
    /// Type of authentication to use when connecting to PuppetDB.
    pub auth: Auth,
}

/// Checks whether the vector of urls contains a url that needs to use SSL, i.e.
/// has `https` as the scheme.
fn is_ssl(server_urls: &Vec<String>) -> bool {
    server_urls.into_iter()
        .any(|url| {
            "https" ==
            Url::parse(&url)
                .unwrap_or_else(|e| pretty_panic!("Error parsing url {:?}: {}", url, e))
                .scheme
        })
}

impl PdbClient {
    pub fn new(config: Config) -> PdbClient {
        let result = if is_ssl(&config.server_urls) {
            PdbClient::with_auth(config)
        } else {
            PdbClient::without_auth(config)
        };

        result.unwrap_or_else(|e| pretty_panic!("Error: {}", e))
    }

    pub fn without_auth(config: Config) -> io::Result<PdbClient> {
        Ok(PdbClient {
            server_urls: config.server_urls,
            auth: Auth::NoAuth,
        })
    }

    #[cfg(not(feature = "puppet-access"))]
    pub fn with_auth(config: Config) -> io::Result<PdbClient> {

        if config.token.is_some() {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                                      "to use token auth please install Puppet Enterprise."));
        }

        if config.cacert.is_none() {
            Err(io::Error::new(io::ErrorKind::InvalidData,
                               "ssl requires 'cacert' to be set"))
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
            } else {
                Err(io::Error::new(io::ErrorKind::InvalidData,
                                   "ssl requires 'cert' and 'key' to be set"))
            }
        }
    }

    #[cfg(feature = "puppet-access")]
    pub fn with_auth(config: Config) -> io::Result<PdbClient> {

        if config.cacert.is_none() {
            Err(io::Error::new(io::ErrorKind::InvalidData,
                               "ssl requires 'cacert' to be set"))
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
            } else if let Some(path) = config.token {
                match puppet_access::read_token(path.clone()) {
                    Ok(contents) => {
                        Ok(PdbClient {
                            server_urls: config.server_urls,
                            auth: Auth::TokenAuth {
                                cacert: config.cacert.unwrap(),
                                token: contents,
                            },
                        })
                    }
                    Err(e) => {
                        Err(io::Error::new(e.kind(),
                                           format!("could not open token {:?}: {}", path, e)))
                    }
                }
            } else {
                let conf_dir = env::home_dir().expect("$HOME directory is not configured");
                let path = puppet_access::default_token_path(conf_dir);
                if !path.is_empty() {
                    match puppet_access::read_token(path.clone()) {
                        Ok(contents) => {
                            Ok(PdbClient {
                                server_urls: config.server_urls,
                                auth: Auth::TokenAuth {
                                    cacert: config.cacert.unwrap(),
                                    token: contents,
                                },
                            })
                        }
                        Err(e) => {
                            match e.kind() {
                                io::ErrorKind::NotFound => {
                                    Err(io::Error::new(io::ErrorKind::NotFound,
                                                       "ssl requires a token, please use `puppet \
                                                        access login` to retrieve a token \
                                                        (alternatively use 'cert' and 'key' for \
                                                        whitelist validation)"))
                                }
                                // For exmaple this could happen if a user made
                                // a directory `$HOME/.puppetlabs/token`
                                _ => {
                                    Err(io::Error::new(e.kind(),
                                                       format!("could not open token {:?}: {}",
                                                               path,
                                                               e)))
                                }
                            }
                        }
                    }
                } else {
                    Err(io::Error::new(io::ErrorKind::Other,
                                       "unable to set default token path, \
                                        please use the `--token` option directly"))
                }
            }
        }
    }

    /// POSTs `query_str` (either AST or PQL) to configured PuppetDBs.
    pub fn query(&self, query_str: String) -> HyperResult {

        let cli = Auth::client(&self.auth);

        let req_body = PdbQueryRequest { query: query_to_json(query_str) }.to_string();

        for server_url in self.server_urls.clone() {
            let req = cli.post(&(server_url + "/pdb/query/v4"))
                .body(&req_body)
                .header(ContentType::json())
                .header(Connection::close());
            let res = Auth::auth_header(&self.auth, req).send();
            if res.is_ok() {
                return res;
            }
        }
        // TODO Collect errors from each server and return them
        let io_error = io::Error::new(io::ErrorKind::ConnectionRefused, "connection refused");
        Err(error::Error::from(io_error))
    }

    /// GETs the trapperkeeper status of each configured PuppetDB and constructs
    /// a map where the keys are the urls and the values are the statuses.
    /// Connection error etc. are represented as JSON objects with a single
    /// `error` key whose value is the error message.
    pub fn status(&self) -> json::Json {
        let mut map = BTreeMap::new();
        let cli = Auth::client(&self.auth);

        for server_url in self.server_urls.clone() {
            let req = cli.get(&(server_url.clone() + "/status/v1/services"))
                .header(Connection::close());
            let res = Auth::auth_header(&self.auth, req).send();
            map.insert(server_url, build_response_json(res));
        }
        json::Json::Object(map)
    }
}

#[test]
#[cfg(not(feature = "puppet-access"))]
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
}

#[test]
#[cfg(feature = "puppet-access")]
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

    let missing_cacert_token_auth = Config {
        server_urls: vec!["foo".to_string()],
        cacert: None,
        cert: None,
        key: None,
        token: Some("bar".to_string()),
    };
    assert!(PdbClient::with_auth(missing_cacert_token_auth).is_err());


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

/// A helper struct to make encoding the json for a PDB query request body
/// easier.
impl PdbQueryRequest {
    fn to_string(&self) -> String {
        json::encode(self).unwrap().to_string()
    }
}

/// Converts a PuppetDB AST or PQL to valid JSON. For a PQL query this just
/// means escaping the string. For an AST query this means parsing the string.
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

fn build_error_json(e: String) -> json::Json {
    let mut error_map = BTreeMap::new();
    error_map.insert("error".to_string(), e.to_json());
    json::Json::Object(error_map)
}

fn build_response_json(resp: HyperResult) -> json::Json {
    match resp {
        Ok(mut r) => {
            match r.status {
                hyper::Ok => {
                    let mut b = json::Builder::new(r.bytes().map(|c| c.unwrap() as char));
                    b.build().unwrap_or_else(|e| {
                        let msg = format!("Unable to build JSON object from server: {}", e);
                        build_error_json(msg)
                    })
                }
                _ => {
                    let mut temp = String::new();
                    let msg = match r.read_to_string(&mut temp) {
                        Err(x) => format!("Unable to read response from server: {}", x),
                        _ => temp,
                    };
                    build_error_json(msg)
                }
            }
        }
        Err(e) => build_error_json(e.to_string()),
    }
}
