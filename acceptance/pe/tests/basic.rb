require 'scooter'
require 'beaker-pe'

test_name "basic validation of puppetdb-cli subcommands" do
  puppet_query_on(client, "--help")
  puppet_db_on(client, "--help")

  step 'setup user and get token with puppet-access' do
    console_dispatcher = Scooter::HttpDispatchers::ConsoleDispatcher.new(master)
    administrator_role = console_dispatcher.get_role_by_name('Administrators')
    user = console_dispatcher.generate_local_user
    console_dispatcher.add_user_to_role(user, administrator_role)
    login_with_puppet_access_on(client, user)
  end

  step 'puppet-db status validation' do
    output = puppet_db_on(client, "status").output.to_s
    status = JSON.parse(output)
    assert_equal("running", status.values.first["puppetdb-status"]["state"], "puppetdb is not running")
  end

  step 'puppet-db export/import and puppet-query validation' do
    dir = client.tmpdir('pdb-cli-basic')

    # Export, add dummy fact, then reimport
    puppet_db_on(client, "export #{dir}/pdb_archive.tgz")
    on(client, "cd #{dir} && tar zxvf pdb_archive.tgz")
    on(client, %(sed -i '/osfamily/i "foo" : "bar",' #{dir}/puppetdb-bak/facts/#{client.hostname}.json))
    on(client, "cd #{dir} && tar zcvf pdb_archive.tgz puppetdb-bak/")
    puppet_db_on(client, "import #{dir}/pdb_archive.tgz")

    query_output = puppet_query_on(client, %('inventory[certname]{facts.foo = "bar"}')).output
    assert_match(client.hostname, query_output)
  end
end
