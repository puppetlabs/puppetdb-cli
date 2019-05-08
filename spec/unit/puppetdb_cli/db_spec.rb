# frozen_string_literal: true

require 'spec_helper'

RSpec.describe 'Running `puppetdb db`' do
  subject { PuppetDBCLI.instance_variable_get(:@db_cmd) }

  context 'when no arguments or options are provided' do
    it 'outputs basic help' do
      expect($stdout).to receive(:puts).with(a_string_matching(/NAME.*USAGE.*COMMANDS.*OPTIONS/m))

      PuppetDBCLI.run(['db'])
    end
  end

  context 'when invoking help' do
    it 'outputs basic help' do
      expect($stdout).to receive(:puts).with(a_string_matching(/NAME.*USAGE.*COMMANDS.*OPTIONS/m))

      expect { PuppetDBCLI.run(['db', '--help']) }.to exit_zero
    end
  end
end
