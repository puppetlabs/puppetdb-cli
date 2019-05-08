# frozen_string_literal: true

# Add the import command
#
# The import command is used to submit an exported archive to PuppetDB
module PuppetDBCLI
  @import_cmd = @db_cmd.define_command do
    name 'import'
    usage 'import [options] <path>'
    summary 'import a PuppetDB archive to PuppetDB'

    run do |opts, args, cmd|
      PuppetDBCLI::Utils.log_command_start cmd.name, opts, args

      if args.count.zero?
        PuppetDBCLI.logger.fatal 'No file path provided'
        exit 1
      elsif args.count > 1
        PuppetDBCLI.logger.fatal 'Only one argument, the path to the export file, is allowed.'
        exit 1
      end

      filename = File.expand_path(args.first)
      PuppetDBCLI.logger.info "Starting import from '#{filename}'"

      client = PuppetDBCLI::Utils.open_client_connection(opts)
      response = client.import(filename)

      exit 1 unless response.success?
    end
  end
end
