# puppetdb-cli

## Usage
Example usage:
~~~bash
$ puppetdb-cli '["from","reports",["extract","certname"]]' \
    --limit=2 --order-by='[{"field":"certname","order":"desc"}]'
[{"certname":"host-999"},{"certname":"host-999"}]
~~~

## Configuration
Example file to place at `~/.puppetlabs/client-tools/puppetdb.conf`:
~~~json
{"default_environment":"prod",
 "environments": {"prod": {"root_url":"https://alpha-rho.local:8081",
                           "ca":"<path to ca.pem>",
                           "cert":"<path to cert .pem>",
                           "key":"<path to private-key .pem>",
                  "dev": {"root_url":"http://localhost:8080"}}}
~~~
