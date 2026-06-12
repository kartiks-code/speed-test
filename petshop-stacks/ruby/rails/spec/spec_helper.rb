$LOAD_PATH.unshift File.join(__dir__, '..', 'lib')

require 'in_memory_petstore_repository'
require 'petstore_errors'

RSpec.configure do |config|
  config.expect_with :rspec do |c|
    c.syntax = :expect
  end
  config.order = :random
end
