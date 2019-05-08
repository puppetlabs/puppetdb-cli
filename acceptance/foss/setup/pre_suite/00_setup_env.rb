# frozen_string_literal: true

require 'beaker'
require 'beaker-puppet'

step 'Install puppet-agent.' do
  install_puppet_agent_on(hosts, puppet_collection: 'puppet6')
end

step 'Install puppetserver.' do
  host = master
  install_package(host, 'puppetserver')
  on(host, 'export PATH=/opt/puppetlabs/bin:$PATH && \
puppet config set certname $(facter fqdn) --section master && \
puppet config set server $(facter fqdn) --section main &&\
puppet config set autosign true --section main &&\
puppetserver ca setup')
  on(host, 'service puppetserver start')
  on(host, '/opt/puppetlabs/bin/puppet agent -t')
end

step 'Install puppetdb.' do
  host = database
  on(host, '/opt/puppetlabs/bin/puppet module install puppetlabs/puppetdb')
  manifest_content = <<~MANIFEST
    class { 'puppetdb': }
    class { 'puppetdb::master::config': }
  MANIFEST

  manifest_path = host.tmpfile('puppetdb_manifest.pp')
  create_remote_file(host, manifest_path, manifest_content)
  on(host, puppet_apply("--detailed-exitcodes #{manifest_path}"), acceptable_exit_codes: [0, 2])
  on(host, '/opt/puppetlabs/bin/puppet agent -t')
end

step 'Run an agent to create the SSL certs' do
  host = master
  on(host, "puppet config set server #{master}")
  on(host, '/opt/puppetlabs/bin/puppet agent -t')
end

def git_ref_to_test
  ENV['SHA'] || 'master'
end

step 'Install the puppetdb cli gem from source' do
  host = master
  git_dir = '/opt/pdb-cli-git'
  pr_remote = <<~PR_REMOTE_CONFIG
    [remote "pr"]
    url = https://github.com/puppetlabs/puppetdb-cli.git
    fetch = +refs/pull/*/head:refs/remotes/pr/*
  PR_REMOTE_CONFIG
  install_package(host, 'ruby')
  install_package(host, 'git-core')
  on(host, 'gem install bundler')
  on(host, "git clone https://github.com/puppetlabs/puppetdb-cli.git #{git_dir}")
  on(host, "echo '#{pr_remote}' >> #{git_dir}/.git/config; cat #{git_dir}/.git/config;")
  on(host, "cd #{git_dir}; git fetch pr; git checkout #{git_ref_to_test}")
  on(host, "cd #{git_dir}; bundle install --path vendor; bundle exec rake build")
  on(host, "cd #{git_dir}; gem install --bindir /opt/puppetlabs/bin pkg/puppetdb_cli-*.gem")
end

step 'Write a config file' do
  host = master
  conf = <<~CONF
    {
      "puppetdb": {
        "server_urls": "https://#{database}:8081",
        "cacert": "/etc/puppetlabs/puppet/ssl/certs/ca.pem",
        "cert": "/etc/puppetlabs/puppet/ssl/certs/#{host}.pem",
        "key": "/etc/puppetlabs/puppet/ssl/private_keys/#{host}.pem"
      }
    }
  CONF
  puts conf
  client_tools_dir = '/etc/puppetlabs/client-tools'
  on(host, "mkdir -p #{client_tools_dir}")
  on(host, "echo '#{conf}' > #{client_tools_dir}/puppetdb.conf")
end
