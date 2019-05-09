# frozen_string_literal: true

module PuppetDBCLI
  module Utils
    # Defaults for puppet-query and puppet-db
    module DefaultOptions
      def self.include_default_options(dsl)
        dsl.flag :v, :version, 'Show version of puppetdb cli tool.' do |_, _|
          puts PuppetDBCLI::VERSION
          exit 0
        end

        dsl.flag :h, :help, 'Show help for this command.' do |_, c|
          puts c.help
          exit 0
        end

        dsl.flag :d, :debug, 'Enable debug output.' do |_, _|
          PuppetDBCLI.logger.enable_debug_mode
        end

        dsl.option :c, :config, 'The path to the PuppetDB CLI config', argument: :required

        dsl.option nil, :urls, 'The urls of your PuppetDB instances (overrides SERVER_URLS).', argument: :required

        dsl.option nil, :cacert, 'Overrides the path for the Puppet CA cert', argument: :required

        dsl.option nil, :cert, 'Overrides the path for the Puppet client cert.', argument: :required

        dsl.option nil, :key, 'Overrides the path for the Puppet client private key.', argument: :required

        dsl.option nil, :token, 'Overrides the path for the RBAC token (PE only).', argument: :required
      end
    end
  end
end
