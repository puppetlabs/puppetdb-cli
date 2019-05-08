# frozen_string_literal: true

require 'spec_helper'

RSpec.describe 'Running `puppetdb query`' do
  subject { PuppetDBCLI.instance_variable_get(:@query_cmd) }

  context 'when no arguments or options are provided' do
    it 'prints an error message' do
      expect($stderr).to receive(:write).with("FATAL: No query provided\n")

      expect { PuppetDBCLI.run(['query']) }.to exit_nonzero
    end
  end

  context 'when invoking help' do
    it 'outputs basic help' do
      expect($stdout).to receive(:puts).with(a_string_matching(/NAME.*USAGE.*COMMANDS.*OPTIONS/m))

      expect { PuppetDBCLI.run(['query', '--help']) }.to exit_zero
    end
  end

  context 'when given a valid query' do
  end
end
