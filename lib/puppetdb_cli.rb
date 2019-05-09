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
  end

  require 'puppetdb_cli/query'
  require 'puppetdb_cli/db'

  @base_cmd.add_command Cri::Command.new_basic_help
end
