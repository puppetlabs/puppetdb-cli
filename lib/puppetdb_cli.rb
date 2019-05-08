# frozen_string_literal: true

require 'cri'
require 'puppetdb'

require 'puppetdb_cli/logger'
require 'puppetdb_cli/utils'
require 'puppetdb_cli/version'

# The top level command for the PuppetDB CLI
module PuppetDBCLI
  def self.run(args)
    @base_cmd.run(args)
  end

  @base_cmd = Cri::Command.define do
    name 'puppet'
    usage 'puppet command [options]'
    summary 'PuppetDB CLI'
    description 'A command line tool for interacting with PuppetDB'
    default_subcommand 'help'

    flag :v, :version, 'Show version of puppetdb cli tool.' do |_, _|
      puts PuppetDBCLI::VERSION
      exit 0
    end

    flag :h, :help, 'Show help for this command.' do |_, c|
      puts c.help
      exit 0
    end

    flag :d, :debug, 'Enable debug output.' do |_, _|
      PuppetDBCLI.logger.enable_debug_mode
    end

    option :c, :config, 'The path to the PuppetDB CLI config', argument: :required

    option nil, :urls, 'The urls of your PuppetDB instances (overrides SERVER_URLS).', argument: :required

    option nil, :cacert, 'Overrides the path for the Puppet CA cert', argument: :required

    option nil, :cert, 'Overrides the path for the Puppet client cert.', argument: :required

    option nil, :key, 'Overrides the path for the Puppet client private key.', argument: :required

    option nil, :token, 'Overrides the path for the RBAC token (PE only).', argument: :required
  end

  require 'puppetdb_cli/query'
  require 'puppetdb_cli/db'

  @base_cmd.add_command Cri::Command.new_basic_help
end
