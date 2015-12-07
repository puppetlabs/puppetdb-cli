# Rust Implementation of a PuppetDB CLI

[![Build status](https://ci.appveyor.com/api/projects/status/bhln68k6pdfixrun?svg=true)](https://ci.appveyor.com/project/ajroetker/rust-puppetdb-cli)

[![Build Status](https://travis-ci.org/ajroetker/rust-puppetdb-cli.svg)](https://travis-ci.org/ajroetker/rust-puppetdb-cli)

## Installation

Using `rustc` 1.4.0 (stable) and `cargo` 0.6.0:

```zsh
<rust-puppetdb-cli>$ export PATH=./target/debug
<rust-puppetdb-cli>$ cargo build
...
<rust-puppetdb-cli>$ puppet-pdb query '["from","nodes",["extract","certname"]]'
[
    { "certname" : "baz.example.com" },
    { "certname" : "bar.example.com" },
    { "certname" : "foo.example.com" }
]
```

## Configuration

The Rust PuppetDB CLI accepts a `--config=<path_to_config>` flag which allows
you to configure your ssl credentials and the location of your PuppetDB.

The format of the config file can be deduced from the following example.

```json
{
    "default_environment" : "prod"
    "environments" : {
        "prod" : {
            "server_urls" : [
                "https://pdb.internal.lan:8081",
                "https://read-pdb.internal.lan:8081"
            ],
            "cacert" : "/path/to/cacert",
            "cert" : "/path/to/cert",
            "key" : "/path/to/private_key"
        },
        "dev" : {
            "server_urls" : [
                "http://127.0.0.1:8080"
            ],
        }
    }
}
```
