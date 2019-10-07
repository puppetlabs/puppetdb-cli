# frozen_string_literal: true

# Adds the export command to the PuppetDB CLI
#
# The export command can be used to create an archive of your PuppetDB database
module PuppetDBCLI
  # Transform and validate a String into a
  # keyword anonymization profile
  class AnonymizationTransformer
    def call(str)
      raise ArgumentError unless str.is_a? String

      str.to_sym.tap do |symbol|
        raise unless %i[none low moderate full].include?(symbol)
      end
    end
  end

  @export_cmd = @db_cmd.define_command do
    name 'export'
    usage 'export [options] <path>'
    summary 'export an archive from PuppetDB'

    option :a, :anonymization, 'Archive anonymization profile (low, moderate, full)',
           default: :none,
           argument: :required,
           transform: AnonymizationTransformer.new

    run do |opts, args, cmd|
      PuppetDBCLI::Utils.log_command_start cmd.name, opts, args

      if args.count.zero?
        PuppetDBCLI.logger.fatal 'No file path provided'
        exit 1
      elsif args.count > 1
        PuppetDBCLI.logger.fatal 'Only one argument, the path where the export file will be written, is allowed.'
        exit 1
      end

      filename = File.expand_path args.first
      PuppetDBCLI.logger.info "Starting export to '#{filename}'"

      client = PuppetDBCLI::Utils.open_client_connection opts
      response = client.export(filename, anonymization_profile: opts[:anonymization])

      exit 1 unless response.success?
    end
  end
end
