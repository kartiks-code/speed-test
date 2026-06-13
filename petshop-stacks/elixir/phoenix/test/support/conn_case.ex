defmodule PetstoreWeb.ConnCase do
  @moduledoc """
  ExUnit case template for controller tests.
  Resets the InMemoryRepository before each test and builds a base Conn.
  """
  use ExUnit.CaseTemplate

  using do
    quote do
      import Plug.Conn
      import Phoenix.ConnTest

      @endpoint PetstoreWeb.Endpoint
    end
  end

  setup do
    Petstore.InMemoryRepository.reset()
    {:ok, conn: Phoenix.ConnTest.build_conn()}
  end
end
