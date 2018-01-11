use std::io::{Write, Read};
use std::fs::File;
use std::path::Path;

use rustc_serialize::json;
use kitchensink::utils::{self,NotEmpty};

pub fn default_config_path() -> String {
    let mut conf_dir = utils::local_client_tools_dir();
    conf_dir.push("puppetdb");
    conf_dir.set_extension("conf");
    conf_dir.to_str().unwrap().to_owned()
}

pub fn global_config_path() -> String {
    let mut path = utils::global_client_tools_dir();
    path.push("puppetdb");
    path.set_extension("conf");
    path.to_str().unwrap().to_owned()
}

pub fn remove_trailing_slash(url: String) -> String {
    let mut trimmed = url.clone();
    if Some('/') == trimmed.chars().last() {
        trimmed.pop();
    }
    trimmed.trim().to_owned()
}

fn split_server_urls(urls: String) -> Vec<String> {
    urls.split(",").map(|u| remove_trailing_slash(u.trim().to_string())).collect()
}

#[test]
fn split_server_urls_works() {
    assert_eq!(vec!["http://localhost:8080".to_string(), "http://foo.bar.baz:9190".to_string()],
               split_server_urls("   http://localhost:8080  ,   http://foo.bar.baz:9190/"
                                     .to_string()))
}

#[derive(RustcDecodable,RustcEncodable,Clone,Debug)]
pub struct Config {
    pub server_urls: Vec<String>,
    pub cacert: String,
    pub cert: Option<String>,
    pub key: Option<String>,
    pub token: Option<String>,
}

pub fn merge_configs(first: PdbConfigSection, second: PdbConfigSection) -> PdbConfigSection {
    PdbConfigSection {
        server_urls: second.server_urls.or(first.server_urls),
        cacert: second.cacert.or(first.cacert),
        cert: second.cert.or(first.cert),
        key: second.key.or(first.key),
        token_file: second.token_file.or(first.token_file),
    }
}

impl Config {
    pub fn load(path: String,
                urls: Option<String>,
                cacert: Option<String>,
                cert: Option<String>,
                key: Option<String>,
                token: Option<String>)
                -> Config {

        let server_urls = urls.not_empty()
            .and_then(|s| Some(split_server_urls(s)));

        // TODO Don't parse config if urls aren't HTTP. This is trivial but it
        // would be best to merge the other auth validation code when
        // constructing the client with this.
        if server_urls.is_some() && cacert.is_some() && cert.is_some() && key.is_some() {
            return Config {
                server_urls: server_urls.unwrap(),
                cacert: cacert.unwrap(),
                cert: cert,
                key: key,
                token: None,
            };
        }

        let file_configs = merge_configs(PdbConfigSection::load(global_config_path()),
                                         PdbConfigSection::load(path));
        let flags_config = PdbConfigSection {
            server_urls: server_urls,
            cacert: cacert,
            cert: cert,
            key: key,
            token_file: token,
        };
        let cfg = merge_configs(file_configs, flags_config);

        // TODO Add tests for Config parsing edge cases
        Config {
            server_urls: cfg.server_urls.unwrap_or(default_server_urls()),
            cacert: cfg.cacert.unwrap_or(utils::default_certificate_file()),
            cert: cfg.cert,
            key: cfg.key,
            // Unfortunately the default token_file is retrieved in the client
            // code due to differences in error messages/code path in OS and PE.
            // To properly consilidate defaulting of configuration we'll
            // probably need to consider some refactoring.
            token: cfg.token_file,
        }
    }
}

#[derive(RustcDecodable,RustcEncodable,Debug)]
pub struct PdbConfigSection {
    server_urls: Option<Vec<String>>,
    cacert: Option<String>,
    cert: Option<String>,
    key: Option<String>,
    token_file: Option<String>,
}

fn default_server_urls() -> Vec<String> {
    vec!["http://127.0.0.1:8080".to_string()]
}

fn empty_pdb_config_section() -> PdbConfigSection {
    PdbConfigSection {
        server_urls: None,
        cacert: None,
        cert: None,
        key: None,
        token_file: None
    }
}

impl PdbConfigSection {
    fn load(path: String) -> PdbConfigSection {
        if !Path::new(&path).exists() {
            return empty_pdb_config_section();
        }
        let mut f = File::open(&path)
            .unwrap_or_else(|e| pretty_panic!("Error opening config {:?}: {}", path, e));
        let mut s = String::new();
        if let Err(e) = f.read_to_string(&mut s) {
            pretty_panic!("Error reading from config {:?}: {}", path, e)
        }
        let json = json::Json::from_str(&s)
            .unwrap_or_else(|e| pretty_panic!("Error parsing config {:?}: {}", path, e));
        PdbConfigSection {
            server_urls: match json.find_path(&["puppetdb", "server_urls"])
                .unwrap_or(&json::Json::Null) {
                &json::Json::Array(ref urls) => {
                    Some(urls.into_iter()
                        .map(|url| url.as_string().unwrap().to_string())
                        .collect::<Vec<String>>())
                }
                &json::Json::String(ref urls) => Some(split_server_urls(urls.clone())),
                &json::Json::Null => None,
                _ => {
                    pretty_panic!("Error parsing config {:?}: server_urls must be an Array or a \
                                   String",
                                  path)
                }
            },
            cacert: json.find_path(&["puppetdb", "cacert"])
                .and_then(|s| s.as_string().and_then(|s| Some(s.to_string()))),
            cert: json.find_path(&["puppetdb", "cert"])
                .and_then(|s| s.as_string().and_then(|s| Some(s.to_string()))),
            key: json.find_path(&["puppetdb", "key"])
                .and_then(|s| s.as_string().and_then(|s| Some(s.to_string()))),
            token_file: json.find_path(&["puppetdb", "token-file"])
                .and_then(|s| s.as_string().and_then(|s| Some(s.to_string()))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;
    use std::io::{Write, Error};
    use std::path::PathBuf;

    extern crate tempdir;
    use self::tempdir::*;

    fn create_temp_path(temp_dir: &TempDir, file_name: &str) -> PathBuf {
        temp_dir.path().join(file_name)
    }

    fn spit(file_path: &str, content: &str) -> Result<(), Error> {
        let mut f = try!(File::create(file_path));
        try!(f.write_all(content.as_bytes()));
        Ok(())
    }

    #[test]
    fn load_test_all_fields() {
        let config = PdbConfigSection {
                server_urls: Some(vec!["http://foo".to_string()]),
                cacert: Some("foo".to_string()),
                cert: Some("bar".to_string()),
                key: Some("baz".to_string()),
                token_file: Some("buzz".to_string()),
        };

        let temp_dir = TempDir::new_in("target", "test-").unwrap();
        let temp_path = create_temp_path(&temp_dir, "testfile.json");
        let path_str = temp_path.as_path().to_str().unwrap();

        // Because the token-file param has a dash instead of an underscore, we
        // can easily encode a PdbConfigSection using the standard json tools so
        // we duplicate the config here.
        let config_string = "{\"puppetdb\":{\"server_urls\":[\"http://foo\"],\"cacert\":\"foo\",\"cert\":\"bar\",\"key\":\"baz\",\"token-file\":\"buzz\"}}";
        spit(path_str, config_string).unwrap();
        let slurped_config = Config::load(path_str.to_string(), None, None, None, None, None);

        let PdbConfigSection { server_urls, cacert, cert, key, token_file } = config;
        assert_eq!(server_urls.unwrap()[0], slurped_config.server_urls[0]);
        assert_eq!(cacert.unwrap(), slurped_config.cacert);
        assert_eq!(cert, slurped_config.cert);
        assert_eq!(key, slurped_config.key);
        assert_eq!(token_file, slurped_config.token)
    }

    fn spit_string(file_path: &str, contents: &str) -> Result<(), Error> {
        let mut f = try!(File::create(file_path));
        try!(f.write_all(contents.as_bytes()));
        Ok(())
    }

    #[test]
    fn load_test_only_urls_vector() {
        let temp_dir = TempDir::new_in("target", "test-").unwrap();
        let temp_path = create_temp_path(&temp_dir, "testfile.json");
        let path_str = temp_path.as_path().to_str().unwrap();

        spit_string(&path_str,
                    "{\"puppetdb\":{\"server_urls\":[\"http://foo\"]}}")
            .unwrap();
        let slurped_config = Config::load(path_str.to_string(), None, None, None, None, None);

        assert_eq!("http://foo", slurped_config.server_urls[0]);
        assert_eq!(None, slurped_config.cert);
        assert_eq!(None, slurped_config.key);
    }

    #[test]
    fn load_test_only_urls_string() {
        let temp_dir = TempDir::new_in("target", "test-").unwrap();
        let temp_path = create_temp_path(&temp_dir, "testfile.json");
        let path_str = temp_path.as_path().to_str().unwrap();

        spit_string(&path_str,
                    "{\"puppetdb\":{\"server_urls\":\"http://foo,https://localhost:8080\"}}")
            .unwrap();
        let slurped_config = Config::load(path_str.to_string(), None, None, None, None, None);

        assert_eq!(vec!["http://foo", "https://localhost:8080"],
                   slurped_config.server_urls);
        assert_eq!(None, slurped_config.cert);
        assert_eq!(None, slurped_config.key);
    }

    #[test]
    fn load_test_only_urls_null() {
        let temp_dir = TempDir::new_in("target", "test-").unwrap();
        let temp_path = create_temp_path(&temp_dir, "testfile.json");
        let path_str = temp_path.as_path().to_str().unwrap();

        spit_string(&path_str, "{\"puppetdb\":{\"server_urls\":null}}").unwrap();
        let slurped_config = Config::load(path_str.to_string(), None, None, None, None, None);

        assert_eq!(vec!["http://127.0.0.1:8080"], slurped_config.server_urls);
        assert_eq!(None, slurped_config.cert);
        assert_eq!(None, slurped_config.key);
    }
}
