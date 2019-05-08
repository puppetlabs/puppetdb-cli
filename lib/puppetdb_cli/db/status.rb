# frozen_string_literal: true

# Add the status command to the PuppetDB CLI
#
# The status command is used to query for all the statuses of the configured PuppetDB's
module PuppetDBCLI
  @status_cmd = @db_cmd.define_command do
    name 'status'
    usage 'status [options]'
    summary 'query the PuppetDB status endpoint for each configured PuppetDB'

    run do |opts, args, cmd|
      PuppetDBCLI::Utils.log_command_start cmd.name, opts, args

      unless args.count.zero?
        PuppetDBCLI.logger.fatal 'status command does not allow arguments'
        exit 1
      end
      client = PuppetDBCLI::Utils.open_client_connection opts

      response = client.status
      puts JSON.pretty_generate(response)
    end
  end
end
