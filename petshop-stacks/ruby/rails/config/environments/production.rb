Rails.application.configure do
  config.eager_load = true
  config.consider_all_requests_local = false
  config.log_level = ENV.fetch('RAILS_LOG_LEVEL', 'info').to_sym

  # Honor RAILS_ALLOWED_HOSTS (comma-separated, set by the benchmark harness).
  # When unset, config.hosts stays empty in production, which leaves Rails host
  # authorization disabled — requests from any Host header are accepted.
  ENV.fetch('RAILS_ALLOWED_HOSTS', '').split(',').map(&:strip).reject(&:empty?).each do |host|
    config.hosts << host
  end
end
