require 'scooter'
require 'beaker-pe'

test_name "attempt to execute query without permissions" do

  teardown do
    puppet_access_on(client, 'delete-token-file')
  end

  step 'setup a user without permissions and get token with puppet-access' do
    console_dispatcher = Scooter::HttpDispatchers::ConsoleDispatcher.new(master)

    empty_permissions = {'permissions' => []}
    no_permissions_role = console_dispatcher.generate_role(empty_permissions)

    user = console_dispatcher.generate_local_user
    console_dispatcher.add_user_to_role(user, no_permissions_role)
    login_with_puppet_access_on(client, user,  {:lifetime => '1d'})
  end

  step 'check that puppet-query fails' do
    puppet_query_on(client, 'nodes[certname]{}', :acceptable_exit_codes => [1]) do |result|
      assert_match("ERROR - [GET /pdb/query/v4][403] getQueryForbidden  Permission denied: User does not have permission to access PuppetDB", result.output)
    end
  end

end
