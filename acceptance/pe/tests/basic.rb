require 'scooter'
require 'beaker-pe'

test_name "basic validation of puppetdb-cli subcommands" do
  puppet_query_on(client, "--help")
  puppet_db_on(client, "--help")

  dir = client.tmpdir('puppetdb-cli')

  teardown do
    client.rm_rf(dir)
  end

  step 'setup user and get token with puppet-access' do
    console_dispatcher = Scooter::HttpDispatchers::ConsoleDispatcher.new(master)
    administrator_role = console_dispatcher.get_role_by_name('Administrators')
    user = console_dispatcher.generate_local_user
    console_dispatcher.add_user_to_role(user, administrator_role)
    login_with_puppet_access_on(client, user)
  end

  step 'puppet-db status validation' do
    puppet_db_on(client, "status") do |result|
      status = JSON.parse(result.output)
      assert_equal("running", status.values.first["puppetdb-status"]["state"], "puppetdb is not running")
    end
  end

  step 'puppet-db export validation' do
    puppet_db_on(client, "export #{dir}/pdb_archive.tgz")

    on(client, "file #{dir}/pdb_archive.tgz") do |file_type|
      assert_match('gzip', file_type.output)
    end
  end

  step 'puppet-db import validation' do
    # Pipe the output to a file since it also includes the archive in binary format
    # TODO do not include the binary payload when running with --log-level debug
    puppet_db_on(client, "import #{dir}/pdb_archive.tgz -l debug > output.txt 2>&1") do |result|
      on(client, %[grep '"ok" : true' output.txt])
    end
  end

  step 'puppet-query validation' do
    puppet_query_on(client, 'nodes[certname]{}') do |result|
      assert_match(client.hostname, result.output)
    end
  end
end
