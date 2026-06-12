Rails.application.configure do
  config.eager_load = true
  config.consider_all_requests_local = false
  config.log_level = ENV.fetch('RAILS_LOG_LEVEL', 'info').to_sym
end
