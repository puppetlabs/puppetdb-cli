require 'json'

# Copies puppet ca from master to host
#
# === Returns
#
# +string+ - path of ca file or nil on fail
def copy_ca_from_master_to(host)
  ca_pem_contents = on(master, 'cat /etc/puppetlabs/puppet/ssl/certs/ca.pem').stdout.chomp
  if (host.platform =~ /win/)
    ca_pem_location       = ['C:', 'Windows', 'Temp', 'ca.pem'].join('\\')
  else
    ca_pem_location       = ['/tmp', 'ca.pem'].join('/')
  end
  res = create_remote_file(host, ca_pem_location, ca_pem_contents) unless master.hostname == host.hostname
  return res.exit_code == 0 ? ca_pem_location : nil
end

step "Install Puppet Enterprise." do
  install_pe
end

step 'copy ca.pem from master to client node' do
  client = find_only_one('client')
  $ca_pem_location = copy_ca_from_master_to(client)
end

step 'create puppet-db/query config file on client node' do
  client = find_only_one('client')

  conf = {
    'puppetdb' => {
      'server_urls' => ["https://#{master.hostname}:8081"],
      'cacert'      => $ca_pem_location
    }
  }
  write_client_tool_config_on(client, 'global', 'db', conf.to_json)
end

step 'create puppet-access config file on client node' do
  client = find_only_one('client')

  conf = {
    'service-url'      => "https://#{master.hostname}:4433/rbac-api",
    'certificate-file' => $ca_pem_location
  }

  write_client_tool_config_on(client, 'global', 'access', conf.to_json)
end

step "Install PE Client Tools" do
  opts = {
    :puppet_collection       => 'PC1',
    :pe_client_tools_sha     => ENV['SHA'],
    :pe_client_tools_version => ENV['SUITE_VERSION'] || ENV['SHA']
  }

  client = find_only_one('client')

  install_pe_client_tools_on(client, opts)
end
