# frozen_string_literal: true

require 'spec_helper'

RSpec.describe PuppetDBCLI::Utils do
  describe '.log_command_start' do
    context 'when not in debug mode' do
      it 'does not print messages' do
        expect(PuppetDBCLI.logger).to receive(:debug).exactly(3).times
        expect($stderr).not_to receive(:write)
        described_class.log_command_start('cmd', {}, {})
      end
    end
  end

  describe '.construct_config_overrides' do
    context 'when no configuration is given' do
      it { expect(subject.construct_config_overrides({})).to be_empty }
    end

    context 'when unrecognized configuration is given' do
      it { expect(subject.construct_config_overrides(not_a_config_option: :foo)).to be_empty }
    end

    context 'when given partial configuiration options' do
      let(:config) do
        {
          urls: 'pdb1.com,pdb2.com',
          token: '/path/token'
        }
      end
      let(:response) do
        {
          # TODO: needs to reject use_ssl confusion
          server_urls: ['pdb1.com', 'pdb2.com'],
          token_file: '/path/token'
        }
      end

      it { expect(subject.construct_config_overrides(config)).to eq(response) }
    end

    context 'when given all configuration options' do
      let(:config) do
        {
          config: '/path',
          urls: 'pdb1.com,pdb2.com',
          key: '/path.key',
          cert: '/path.cert',
          cacert: '/path.ca',
          token: '/path/token'
        }
      end
      let(:response) do
        {
          # TODO: needs to reject use_ssl confusion
          config_file: '/path',
          server_urls: ['pdb1.com', 'pdb2.com'],
          key: '/path.key',
          cert: '/path.cert',
          cacert: '/path.ca',
          token_file: '/path/token'
        }
      end

      it { expect(subject.construct_config_overrides(config)).to eq(response) }
    end
  end

  describe '.open_client_connection' do
    before :each do
      expect(subject).to receive(:construct_config_overrides).and_call_original
      expect(PuppetDB::Client).to receive(:new).and_call_original
    end

    context 'with valid config' do
      let(:config) do
        {
          urls: 'http://pdb1.com,http://pdb2.com'
        }
      end

      it { expect(subject.open_client_connection({})).to be_a(PuppetDB::Client) }

      it { expect(subject.open_client_connection(config)).to be_a(PuppetDB::Client) }
    end
  end
end
