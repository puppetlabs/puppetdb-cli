def get_package_version(host, version = nil)

  if version == 'latest'
    return 'latest'
  elsif version.nil?
    version = PuppetDBExtensions.config[:package_build_version].to_s
  end

  # version can look like:
  #   3.0.0
  #   3.0.0.SNAPSHOT.2015.07.08T0945

  # Rewrite version if its a SNAPSHOT in rc form
  if version.include?("SNAPSHOT")
    version = version.sub(/^(.*)\.(SNAPSHOT\..*)$/, "\\1-0.1\\2")
  else
    version = version + "-1"
  end

  ## These 'platform' values come from the acceptance config files, so
  ## we're relying entirely on naming conventions here.  Would be nicer
  ## to do this using lsb_release or something, but...
  if host['platform'].include?('el-5')
    "#{version}.el5"
  elsif host['platform'].include?('el-6')
    "#{version}.el6"
  elsif host['platform'].include?('el-7')
    "#{version}.el7"
  elsif host['platform'].include?('fedora')
    version_tag = host['platform'].match(/^fedora-(\d+)/)[1]
    "#{version}.fc#{version_tag}"
  elsif host['platform'].include?('ubuntu') or host['platform'].include?('debian')
    "#{version}puppetlabs1"
  else
    raise ArgumentError, "Unsupported platform: '#{host['platform']}'"
  end
end

install_opts = options.merge( { :dev_builds_repos => ["PC1"] })
repo_config_dir = 'tmp/repo_configs'

step "Install puppet-agent." do
  install_puppet_agent_on(hosts)
end

step "Install puppetserver." do
  host = master
  install_package(host, 'puppetserver')
  on(host, 'export PATH=/opt/puppetlabs/bin:$PATH && \
puppet config set certname $(facter fqdn) --section master && \
puppet config set server $(facter fqdn) --section main &&\
puppet cert generate $(facter fqdn) --dns_alt_names  $(hostname -s),$(facter fqdn) \
  --ca_name "Puppet CA generated on $(facter fqdn) at $(date \'+%Y-%m-%d %H:%M:%S %z\')"
')
  on(host, 'service puppetserver start')
  on(host, '/opt/puppetlabs/bin/puppet agent -t')
  on(host, 'service puppetserver stop')
end

step "Install puppetdb." do
  host = database
  on(host, '/opt/puppetlabs/bin/puppet module install puppetlabs/puppetdb')
  manifest_content = <<-EOS
    class { 'puppetdb::globals':
      version => '#{get_package_version(host, "4.1.0")}'
    }
    class { 'puppetdb': }
    class { 'puppetdb::master::config': }
    EOS

  manifest_path = host.tmpfile("puppetdb_manifest.pp")
  create_remote_file(host, manifest_path, manifest_content)
  on(host, puppet_apply("--detailed-exitcodes #{manifest_path}"), :acceptable_exit_codes => [0,2])
end

step "Setup puppet-client-tools repositories." do
  install_puppetlabs_dev_repo(master,
                              'puppet-client-tools',
                              ENV['SHA'],
                              repo_config_dir,
                              install_opts)
end

step "Install puppet-client-tools." do
  host = master
  install_package(host, 'puppet-client-tools')
  on(host, "/opt/puppetlabs/bin/puppet-query --help")
  on(host, "/opt/puppetlabs/bin/puppet-query 'nodes{}'")
end
