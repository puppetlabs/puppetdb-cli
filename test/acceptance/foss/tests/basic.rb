test_name "basic validation of puppetdb-cli subcommands" do
  host = master
  on(host, "/opt/puppetlabs/bin/puppet-query --help")
  on(host, "/opt/puppetlabs/bin/puppet-db --help")

  on(host, "/opt/puppetlabs/bin/puppet-query 'nodes{}'")
  on(host, "/opt/puppetlabs/bin/puppet-db status")

  dir = create_tmpdir_on(host)
  on(host, "/opt/puppetlabs/bin/puppet-db export #{dir}/pdb_archive.tgz")
  on(host, "/opt/puppetlabs/bin/puppet-db import #{dir}/pdb_archive.tgz")
end
