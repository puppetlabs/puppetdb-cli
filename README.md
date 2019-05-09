# puppetdb-cli

[![Build status](https://ci.appveyor.com/api/projects/status/ip998jin18j4g0yv?svg=true)](https://ci.appveyor.com/project/puppetlabs/puppetdb-cli)
[![Build Status](https://travis-ci.org/puppetlabs/puppetdb-cli.svg)](https://travis-ci.org/puppetlabs/puppetdb-cli)

> **Note**: This repository is still under active development. Stay tuned for
> release dates and functionality changes.

The PuppetDB CLI project provides Puppet subcommands for querying PuppetDB data,
via `puppet query <query>`, and PuppetDB administrative tasks, `puppet db
<import|export|status>`. The `query` subcommand will allow you to query PuppetDB
using either PQL or AST syntax. The `db` subcommand is a replacement for the older
`puppetdb <export|import>` commands with faster startup times and much
friendlier error messages.

## Compatibility

This CLI is compatible with
[PuppetDB 4.0.0](https://docs.puppetlabs.com/puppetdb/4.0/release_notes.html#section)
and greater.

## Installation

### Prerequisites

* Ruby

### Installation from rubygems

The PuppetDB CLI can be installed via a `gem install`.

```bash
gem install --bindir /opt/puppetlabs/bin puppetdb_cli
```

If the machine does not have Puppet installed, you can simply use `gem install puppetdb_cli`
and use the `puppet-query` and `puppet-db` executables directly.

### Installation from source

From the cloned repository

```bash
bundle exec rake install
```

## Usage

Example usage:

```bash
$ puppet-query 'nodes[certname]{}'
[
  {
    "certname" : "baz.example.com"
  },
  {
    "certname" : "bar.example.com"
  },
  {
    "certname" : "foo.example.com"
  }
]
$ puppet-db status
{
  "puppetdb-status": {
    "service_version": "4.0.0-SNAPSHOT",
    "service_status_version": 1,
    "detail_level": "info",
    "state": "running",
    "status": {
      "maintenance_mode?": false,
      "queue_depth": 0,
      "read_db_up?": true,
      "write_db_up?": true
    }
  },
  "status-service": {
    "service_version": "0.3.1",
    "service_status_version": 1,
    "detail_level": "info",
    "state": "running",
    "status": {}
  }
}
```

## Configuration

The Rust PuppetDB CLI accepts a `--config=<path_to_config>` flag which allows
you to configure your ssl credentials and the location of your PuppetDB.

By default the tool will use `$HOME/.puppetlabs/client-tools/puppetdb.conf` as
it's configuration file if it exists. You can also configure a global
configuration (for all users) in `/etc/puppetlabs/client-tools/puppetdb.conf`
(`C:\ProgramData\puppetlabs\client-tools\puppetdb.conf` on Windows) to fall back
to if the per-user configuration is not present.

The format of the config file can be deduced from the following example.

```json
  {
    "puppetdb" : {
      "server_urls" : [
        "https://<PUPPETDB_HOST>:8081",
        "https://<PUPPETDB_REPLICA_HOST>:8081"
      ],
      "cacert" : "/path/to/cacert",
      "cert" : "/path/to/cert",
      "key" : "/path/to/private_key",
      "token-file" : "/path/to/token (PE only)"
      },
    }
  }
```
