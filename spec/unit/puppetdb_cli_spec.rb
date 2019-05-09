# frozen_string_literal: true

require 'spec_helper'

RSpec.describe PuppetDBCLI do
  it 'has a version number' do
    expect(PuppetDBCLI::VERSION).not_to be nil
  end
end
