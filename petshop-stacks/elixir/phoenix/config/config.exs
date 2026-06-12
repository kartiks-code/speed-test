import Config

config :petstore, PetstoreWeb.Endpoint,
  http: [ip: {0, 0, 0, 0}, port: 8080],
  render_errors: [
    formats: [json: PetstoreWeb.ErrorJSON],
    layout: false
  ],
  pubsub_server: Petstore.PubSub,
  live_view: [signing_salt: "petstore_lv"]

config :petstore, :repository, Petstore.PostgresRepository

config :logger, :console,
  format: "$time $metadata[$level] $message\n",
  metadata: [:request_id]

config :phoenix, :json_library, Jason

import_config "#{config_env()}.exs"
