test_name "basic validation of puppetdb-cli subcommands" do
  on(host, "/opt/puppetlabs/bin/puppet-query --help")
  on(host, "/opt/puppetlabs/bin/puppet-db --help")
  on(host, "/opt/puppetlabs/bin/puppet-query 'nodes{}'")
end
