# frozen_string_literal: true

require 'spec_helper'

RSpec.describe 'Running `puppetdb db`' do
  subject { PuppetDBCLI.instance_variable_get(:@db_cmd) }

  context 'when given the debug flag' do
    it 'enables debug mode' do
      expect(PuppetDBCLI.logger).to receive(:enable_debug_mode).and_call_original
      expect($stderr).to receive(:write).with(a_string_matching(/\[.*\] DEBUG -- Debug mode enabled/))
      expect($stdout).to receive(:puts).with(a_string_matching(/NAME.*USAGE.*SUBCOMMANDS.*OPTIONS/m))
      PuppetDBCLI.run(['db', '--debug'])
    end
  end

  context 'when invoking version' do
    it 'prints the version' do
      expect($stdout).to receive(:puts).with(PuppetDBCLI::VERSION)

      expect { PuppetDBCLI.run(['db', '--version']) }.to exit_zero
    end
  end

  context 'when no arguments or options are provided' do
    it 'outputs basic help' do
      expect($stdout).to receive(:puts).with(a_string_matching(/NAME.*USAGE.*COMMANDS.*OPTIONS/m))

      PuppetDBCLI.run(['db'])
    end
  end

  context 'when invoking help' do
    it 'outputs basic help' do
      expect($stdout).to receive(:puts).with(a_string_matching(/NAME.*USAGE.*SUBCOMMANDS.*OPTIONS/m))

      expect { PuppetDBCLI.run(['db', '--help']) }.to exit_zero
    end
  end
end
