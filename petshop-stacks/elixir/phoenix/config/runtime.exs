import Config

if System.get_env("DATABASE_URL") do
  config :petstore, Petstore.PostgresRepository,
    url: System.get_env("DATABASE_URL"),
    pool_size: String.to_integer(System.get_env("POOL_SIZE", "10"))
else
  config :petstore, Petstore.PostgresRepository,
    hostname: System.get_env("POSTGRES_HOST", "localhost"),
    port: String.to_integer(System.get_env("POSTGRES_PORT", "5434")),
    database: System.get_env("POSTGRES_DB", "elixir-phoenix"),
    username: System.get_env("POSTGRES_USER", "myuser"),
    password: System.get_env("POSTGRES_PASSWORD", "mypassword"),
    pool_size: String.to_integer(System.get_env("POOL_SIZE", "10"))
end

if System.get_env("SECRET_KEY_BASE") do
  config :petstore, PetstoreWeb.Endpoint,
    secret_key_base: System.get_env("SECRET_KEY_BASE")
else
  config :petstore, PetstoreWeb.Endpoint,
    secret_key_base: "dev_secret_key_base_needs_64_chars_minimum_please_change_me_xxx"
end

if System.get_env("PHX_SERVER") do
  config :petstore, PetstoreWeb.Endpoint, server: true
end
