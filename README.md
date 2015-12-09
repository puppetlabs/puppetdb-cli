# puppetdb-cli

## Usage

Example usage:

~~~bash

    $ git submodule update --init
    $ mkdir build && cd build
    $ CMAKE_PREFIX_PATH=/usr/local/opt/curl/lib cmake ..
    $ make -j
    $ ./bin/puppet-db query '["from","reports",["extract","certname"]]'
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
