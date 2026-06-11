defmodule Petstore.Application do
  use Application

  @impl true
  def start(_type, _args) do
    children = base_children() ++ repo_children(Application.get_env(:petstore, :repository))

    opts = [strategy: :one_for_one, name: Petstore.Supervisor]
    Supervisor.start_link(children, opts)
  end

  defp base_children do
    [
      {Phoenix.PubSub, name: Petstore.PubSub},
      PetstoreWeb.Endpoint
    ]
  end

  defp repo_children(Petstore.PostgresRepository) do
    db_config = Application.get_env(:petstore, Petstore.PostgresRepository, [])

    opts =
      if url = db_config[:url] do
        [name: Petstore.DB, url: url, pool_size: db_config[:pool_size] || 10]
      else
        [
          name: Petstore.DB,
          hostname: db_config[:hostname] || "localhost",
          port: db_config[:port] || 5434,
          database: db_config[:database] || "elixir-phoenix",
          username: db_config[:username] || "myuser",
          password: db_config[:password] || "mypassword",
          pool_size: db_config[:pool_size] || 10
        ]
      end

    [{Postgrex, opts}]
  end

  defp repo_children(Petstore.InMemoryRepository) do
    [{Petstore.InMemoryRepository, []}]
  end

  defp repo_children(_), do: []
end
