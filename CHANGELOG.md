# Change Log
All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## [1.0.0] - 2016-04-07

### Summary

Initial release of the PuppetDB CLI subcommands.

The PuppetDB CLI is intended to facilitate friendlier interactions with the
PuppetDB API. 

The PuppetDB CLI accepts a configuration file with SSL credentials and the url
for your PuppetDB server so you can issue queries to PuppetDB on your own
machine without needing to type long `curl` invocations.

We intend to use the PuppetDB CLI to provide human readable output formats and
helpful hints for interacting with the API more generally.

### Installation

Please see the
[PuppetDB documentation](https://docs.puppetlabs.com/puppetdb/master/pdb_client_tools.html)
for installation and usage instructions.

### Features

- New implementations of our `puppetdb import` and `puppetdb export` tools for
  faster startup. The commands are now Puppet subcommands `puppet-db import` and
  `puppet-db export` respectively.
- A `puppet-query` subcommand for querying PuppetDB with PQL or AST queries.

### Contributors 

Andrew Roetker, Rob Browing, Ryan Senior, and Wyatt Alt.
