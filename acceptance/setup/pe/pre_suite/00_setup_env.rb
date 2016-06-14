install_opts = options.merge( { :dev_builds_repos => ["PC1"] })
repo_config_dir = 'tmp/repo_configs'

step "Install Puppet Enterprise." do
  install_pe
end

step "Setup pe-client-tools repositories." do
  install_puppetlabs_dev_repo(master,
                              'pe-client-tools',
                              ENV['SHA'],
                              repo_config_dir,
                              install_opts)
end

step "Install pe-client-tools." do
  host = master
  install_package(host, 'pe-client-tools')
end
