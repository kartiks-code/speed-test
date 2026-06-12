import Config

config :petstore, :repository, Petstore.InMemoryRepository

config :petstore, PetstoreWeb.Endpoint,
  http: [ip: {127, 0, 0, 1}, port: 4002],
  secret_key_base: "test_secret_key_base_at_least_64_chars_long_padding_here_xxxxx",
  server: false

config :logger, level: :warning

config :phoenix, :plug_init_mode, :runtime
