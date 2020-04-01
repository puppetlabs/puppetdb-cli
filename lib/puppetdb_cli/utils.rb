# frozen_string_literal: true

require 'puppetdb_cli/utils/default_options'

# Utils for PuppetDBCLI
#
# Primarily used for interaction with the PuppetDB::Client
module PuppetDBCLI::Utils
  def self.log_command_start(name, opts, args)
    PuppetDBCLI.logger.debug "Running the #{name} command"
    PuppetDBCLI.logger.debug "CLI options: #{opts}"
    PuppetDBCLI.logger.debug "CLI arguments: #{args.to_a}"
  end

  def self.construct_config_overrides(cli_opts)
    {
      config_file: cli_opts[:config],
      server_urls: cli_opts[:urls]&.split(','),
      key: cli_opts[:key],
      cert: cli_opts[:cert],
      cacert: cli_opts[:cacert],
      pem: {
        'key'     => cli_opts[:key],
        'cert'    => cli_opts[:cert],
        'ca_file' => cli_opts[:cacert],
      },
      token_file: cli_opts[:token]
    }.delete_if { |_, v| v.nil? }
  end

  def self.open_client_connection(cli_opts)
    config_overrides = construct_config_overrides cli_opts
    PuppetDBCLI.logger.debug "Initializing client connection with configuration overrides: #{config_overrides}"

    PuppetDB::Client.new(config_overrides)
  rescue URI::InvalidURIError => e
    PuppetDBCLI.logger.fatal "The provided PuppetDB server url was invalid. Failed with message '#{e.message}'"
    exit 1
  # This will catch errors like SocketError from HTTParty and RuntimeError from puppetdb-ruby
  rescue RuntimeError => e
    PuppetDBCLI.logger.fatal e.message
    exit 1
  end

  def self.send_query(client, query)
    PuppetDBCLI.logger.debug "Sending query request '#{query}'"

    client.request('', query)
  rescue SocketError => e
    PuppetDBCLI.logger.fatal e.message
    exit 1
  rescue PuppetDB::APIError => e
    puts e.response
    PuppetDBCLI.logger.fatal "Last PuppetDB API response code #{e.response&.code}"
    exit 1
  end
end
