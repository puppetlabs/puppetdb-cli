# frozen_string_literal: true

# Add the db command
#
# This subcommand has no functionalty other than --help. It's purpose is to contain
# subcommands for compatibility with usage as 'puppet db'.
module PuppetDBCLI
  @db_cmd = @base_cmd.define_command do
    name 'db'
    usage 'db [options] <subcommand>'
    summary 'manage PuppetDB administrative tasks'
    default_subcommand 'help'

    flag :h, :help, 'Show help for this command.' do |_, c|
      puts c.help
      exit 0
    end
  end

  @db_cmd.add_command Cri::Command.new_basic_help

  require 'puppetdb_cli/db/import'
  require 'puppetdb_cli/db/export'
  require 'puppetdb_cli/db/status'
end
