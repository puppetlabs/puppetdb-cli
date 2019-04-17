# frozen_string_literal: true

require 'logger'

# PuppetDB CLI namespace
module PuppetDBCLI
  def self.logger
    @logger ||= PuppetDBCLI::Logger.new
  end

  # A logger for the PuppetDB CLI
  #
  # Overrides standard format of logs for better cli ouput, but reverts to traditional
  # log formatting when in debug mode
  class Logger < ::Logger
    def initialize
      super($stderr)

      self.formatter = proc do |severity, datetime, _progname, msg|
        if level == ::Logger::DEBUG
          "[#{datetime.strftime '%Y-%m-%d %H:%M:%S.%6N'}] #{severity} -- #{msg}\n"
        else
          "#{severity}: #{msg}\n"
        end
      end

      self.level = ::Logger::INFO
    end

    def enable_debug_mode
      self.level = ::Logger::DEBUG
      debug 'Debug mode enabled'
    end
  end
end
