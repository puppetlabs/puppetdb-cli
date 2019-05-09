# frozen_string_literal: true

require 'bundler/setup'
require 'puppetdb_cli'

RSpec.configure do |config|
  # Enable flags like --only-failures and --next-failure
  config.example_status_persistence_file_path = '.rspec_status'

  # Disable RSpec exposing methods globally on `Module` and `main`
  config.disable_monkey_patching!

  config.before :each do
    PuppetDBCLI.logger.level = ::Logger::INFO
  end

  config.expect_with :rspec do |c|
    c.syntax = :expect
  end
end

RSpec::Matchers.define(:exit_with_status) do |expected_status|
  supports_block_expectations

  match do |block|
    expectation_passed = false

    begin
      block.call
    rescue SystemExit => e
      expectation_passed = values_match?(expected_status, e.status)
    rescue StandardError
      nil
    end

    expectation_passed
  end
end

RSpec::Matchers.define(:exit_zero) do
  supports_block_expectations

  match do |block|
    expect { block.call }.to exit_with_status(0)
  end
end

RSpec::Matchers.define(:exit_nonzero) do
  supports_block_expectations

  match do |block|
    expect { block.call }.to exit_with_status(be_nonzero)
  end
end
