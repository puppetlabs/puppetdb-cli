# puppetdb-cli

> ⚠️ This repo has been superseded by https://github.com/puppetlabs/pe-cli

The PuppetDB CLI project provide Puppet subcommands for querying PuppetDB data,
via `puppet query <query>`, and PuppetDB administrative tasks, `puppet db
<import|export|status>`. The `query` subcommand will allow you to query PuppetDB
using either the upcoming PQL syntax of the traditional PuppetDB query syntax
(also known as AST). The `db` subcommand is a replacement for the older
`puppetdb <export|import>` commands with faster startup times and much
friendlier error messages.

## Compatibility

This CLI is compatible with
[PuppetDB 4.0.0](https://docs.puppetlabs.com/puppetdb/4.0/release_notes.html#section)
and greater.

## Installation

Please see
[the PuppetDB documentation](https://docs.puppet.com/puppetdb/latest/pdb_client_tools.html)
for instructions on how to install the `puppet-client-tools` package.

## Installation from source

TBD

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

The Go PuppetDB CLI accepts a `--config=<path_to_config>` flag which allows
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
