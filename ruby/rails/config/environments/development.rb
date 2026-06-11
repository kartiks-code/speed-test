Rails.application.configure do
  config.eager_load = false
  config.consider_all_requests_local = true
  # Allow requests from any host so Docker container name resolution works in benchmarks.
  config.hosts.clear
end
