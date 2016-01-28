# puppetdb-cli

> **Note**: This repository is still under active development. Stay tuned for
> release dates and functionality changes.

The PuppetDB CLI project provide Puppet subcommands for querying PuppetDB data,
via `puppet query <query>`, and PuppetDB administrative tasks, `puppet db
<import|export|status>`. The `query` subcommand will allow you to query PuppetDB
using either the upcoming PQL syntax of the traditional PuppetDB query syntax
(also known as AST). The `db` subcommand is a replacement for the older
`puppetdb <export|import>` commands with faster startup times and much
friendlier error messages.

## Usage

Example usage:

~~~bash

    $ puppet-query '["from","reports",["extract","certname"]]'
    [{"certname":"host-1"}]
 
~~~

## Configuration

Example file to place at `~/.puppetlabs/client-tools/puppetdb.conf`:

~~~json

    {
        "default_environment":"prod",
        "environments":{
            "prod":{
                "server_urls":[ "https://alpha-rho.local:8081" ],
                "ca":"<path to ca.pem>",
                "cert":"<path to cert .pem>",
                "key":"<path to private-key .pem>"
            },
            "dev":{
                "server_urls":[ "http://localhost:8080" ]
            }
        }
    }

~~~

## Install from source

Make sure the appropriate version of leatherman is installed on your system.

~~~bash

      $ mkdir build
      $ cmake build
      $ make -C build -j2
 
~~~

if you don't want to install leatherman globally or need to build with a
different version of curl or openssl (looking at you OSX) you can use
`CMAKE_PREFIX_PATH=<paths to deps> cmake build` in the steps above.
