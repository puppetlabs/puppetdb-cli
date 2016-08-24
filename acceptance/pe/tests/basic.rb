require 'scooter'

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

  puppet_query_on(client, "nodes{}")
  puppet_db_on(client, "status")
  dir = client.tmpdir('pdb-cli-basic')
  puppet_db_on(client, "export #{dir}/pdb_archive.tgz")
  puppet_db_on(client, "import #{dir}/pdb_archive.tgz")
end
