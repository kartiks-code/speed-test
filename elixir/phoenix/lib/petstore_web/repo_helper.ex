defmodule PetstoreWeb.RepoHelper do
  @moduledoc "Returns the configured repository module."

  def repo do
    Application.get_env(:petstore, :repository, Petstore.PostgresRepository)
  end
end
