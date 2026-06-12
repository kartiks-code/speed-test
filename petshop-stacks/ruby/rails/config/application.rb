require_relative 'boot'

require 'rails'
require 'action_controller/railtie'
require 'active_support/railtie'

Bundler.require(*Rails.groups)

module Petstore
  class Application < Rails::Application
    config.api_only = true
    config.autoload_paths << config.root.join('lib')
    config.logger = ActiveSupport::Logger.new($stdout)
    config.log_level = :info
  end
end
