max_threads_count = ENV.fetch('RAILS_MAX_THREADS', 5)
min_threads_count = ENV.fetch('RAILS_MIN_THREADS') { max_threads_count }
threads min_threads_count, max_threads_count

port ENV.fetch('PORT', 8080)
environment ENV.fetch('RAILS_ENV', 'development')

# Cluster mode: 0 (default) keeps single-mode behavior.
workers ENV.fetch('WEB_CONCURRENCY', 0).to_i

# Puma enables preload by default in cluster mode. Preload is kept off because
# the naive benchmark variant runs in the development env, where preload
# conflicts with the code reloader. It is also safe (just not required) with
# 2 workers in production; enabling preload there is a possible future tweak
# for copy-on-write memory savings.
preload_app! false

plugin :tmp_restart
