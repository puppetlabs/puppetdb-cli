# PuppetDB CLI

## `puppet-db`
> Hoo, boy, naming. Is the worst.

~~~
NAME
   puppet-db - query data from PuppetDB
   pdb
~~~

## `Synopsis`

```
$ puppet-db <scope> <query> [options]
```

## `Options`

> Enumerate usage of all arguments and flags to the program

```shell
Global options:
   -h
  --help                      Show this message
  --verbose                   Set verbose output
   -V
  --version                   Display version information
   -d
  --debug                     Enable debug logging.
  --service-url               https://<your.puppetdb.server>:8081/pdb/query/v4/
  --tlsv1
  --ca-cert                   Defaults to
                              /etc/puppet/ssl/certs/ca.pem
  --node-cert                 Defaults to
                              /etc/puppet/ssl/certs/<node>.pem
  --node-key                  Defaults to
                              /etc/puppet/ssl/private_keys/<node>.pem
  --ast                       Use old-skool AST syntax
  --file-with-json            Text file containing AST query
   -f
  --file                      Text file containing PQL query
   -l
  --limit <int>
   -o
  --offset <int>
   -b
  --order-by <string>
   -F
  --format <csv,json,yaml>
  --csv
  --json
  --yaml
```

## `Actions`

> Provide a detailed description of each action (subcommand) to your program.

```shell
```

## `Environment`

> List all environment variables that affect your program

```
```

## `Examples`

> Show how to successfully use the progam. Describe common cases first, and more elaborate operations after.

```
```

## `Diagnostics`

> Describe various error conditions and how to resolve them

```
```

## See also

```
```
