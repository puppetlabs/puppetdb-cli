# frozen_string_literal: true

require 'json'

# Add the query command to PuppetDBCLI
#
# The query command submits queries to /pdb/query/v4
module PuppetDBCLI
  @query_cmd = @base_cmd.define_command do
    name 'query'
    usage 'query [options] <query>'
    summary 'Query puppetdb with AST or PQL'

    flag :h, :help, 'Show help for this command.' do |_, c|
      c.add_command Cri::Command.new_basic_help
      puts c.help
      exit 0
    end

    run do |opts, args, cmd|
      PuppetDBCLI::Utils.log_command_start cmd.name, opts, args

      if args.count.zero?
        PuppetDBCLI.logger.fatal 'No query provided'
        exit 1
      elsif args.count > 1
        PuppetDBCLI.logger.fatal 'More than one argument provided. Try wrapping the query in single quotes.'
        exit 1
      end
      query = args.first

      client = PuppetDBCLI::Utils.open_client_connection opts

      response = PuppetDBCLI::Utils.send_query client, query
      puts JSON.pretty_generate(response.data)
    end
  end
end
