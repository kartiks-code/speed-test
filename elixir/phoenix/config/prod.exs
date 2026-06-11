import Config

# Production config (runtime config is in runtime.exs)
config :petstore, PetstoreWeb.Endpoint,
  cache_static_manifest: "priv/static/cache_manifest.json",
  server: true
