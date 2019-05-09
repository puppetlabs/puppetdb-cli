# frozen_string_literal: true

# Add the db command
#
# This subcommand has no functionalty other than --help. It's purpose is to contain
# subcommands for compatibility with usage as 'puppet db'.
module PuppetDBCLI
  @db_cmd = @base_cmd.define_command do |dsl|
    dsl.name 'db'
    dsl.usage 'db [options] <subcommand>'
    dsl.summary 'manage PuppetDB administrative tasks'
    dsl.default_subcommand 'help'

    PuppetDBCLI::Utils::DefaultOptions.include_default_options(dsl)
  end

  @db_cmd.add_command Cri::Command.new_basic_help

  require 'puppetdb_cli/db/import'
  require 'puppetdb_cli/db/export'
  require 'puppetdb_cli/db/status'
end
