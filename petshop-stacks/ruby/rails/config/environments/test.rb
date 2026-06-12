Rails.application.configure do
  config.eager_load = false
  config.consider_all_requests_local = true
  config.cache_store = :null_store
end
