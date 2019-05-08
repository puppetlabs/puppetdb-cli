# frozen_string_literal: true

require 'spec_helper'

RSpec.describe PuppetDBCLI do
  it 'has a version number' do
    expect(PuppetDBCLI::VERSION).not_to be nil
  end

  context 'when invoking help' do
    it 'outputs basic help' do
      expect($stdout).to receive(:puts).with(a_string_matching(/NAME.*USAGE.*DESCRIPTION.*COMMANDS.*OPTIONS/m))

      expect { described_class.run(['--help']) }.to exit_zero
    end
  end

  context 'when given the debug flag' do
    it 'enables debug mode' do
      expect(PuppetDBCLI.logger).to receive(:enable_debug_mode).and_call_original
      expect($stderr).to receive(:write).with(a_string_matching(/\[.*\] DEBUG -- Debug mode enabled/))
      expect($stdout).to receive(:puts).with(a_string_matching(/NAME.*USAGE.*DESCRIPTION.*COMMANDS.*OPTIONS/m))
      described_class.run(['--debug'])
    end
  end

  context 'when invoking version' do
    it 'prints the version' do
      expect($stdout).to receive(:puts).with(PuppetDBCLI::VERSION)

      expect { described_class.run(['--version']) }.to exit_zero
    end
  end
end
