# frozen_string_literal: true

require 'json'

# Add the query command to PuppetDBCLI
#
# The query command submits queries to /pdb/query/v4
module PuppetDBCLI
  @query_cmd = @base_cmd.define_command do |dsl|
    dsl.name 'query'
    dsl.usage 'query [options] <query>'
    dsl.summary 'Query puppetdb with AST or PQL'

    PuppetDBCLI::Utils::DefaultOptions.include_default_options(dsl)

    dsl.run do |opts, args, cmd|
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
