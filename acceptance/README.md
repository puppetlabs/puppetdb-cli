# Running puppetdb-cli Acceptance Tests

## Running tests with PE
### Prerequisites
* ruby
* bundler
* redis-cli

### Installing gem dependencies
Run `bundle` to install missing gem dependencies. Most of the gem paths can be overridden.

The examples below are all valid ways to provide custom beaker versions:

```
export BEAKER_LOCATION=file:///path/to/beaker
export BEAKER_LOCATION=git://github.com/puppetlabs/beaker#master
export BEAKER_LOCATION=4.16.0
```

Check out the Gemfile for other gems that can be overridden this way.

### Environment variables
* `SHA` - sha or tag that exists on builds.delivery.puppetlabs.net/pe-client-tools
* `SUITE_VERSION` - the `git describe` part of the artifact filename (if the complete filename is `pe-client-tools-19.8.1.20.g9109bb4-x64.msi`, the variable should be `19.8.1.20.g9109bb4`); note that platforms that use repos for installation (el/deb) do not seem to require this variable
* `PE_FAMILY` - determines what stream of PE will be used to test against (can use `master`)

### Running tests

After setting the variables above, the tests can be run with the following command:

```
bundle exec rake acceptance
```

### Re-running or debugging failing tests

If something happens and tests fail, they can be manually rerun with `bundle
exec beaker exec tests`, provided that the hosts are still available.

Make sure to run `bundle exec beaker destroy` when you're finished in order to
remove any provisioned hosts.
