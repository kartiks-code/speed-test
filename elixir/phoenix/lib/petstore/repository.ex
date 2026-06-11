defmodule Petstore.Repository do
  @moduledoc """
  Behaviour defining the Petstore persistence interface.
  Implementations: Petstore.PostgresRepository (production), Petstore.InMemoryRepository (tests).
  """

  # Pet operations
  @callback add_pet(map()) :: {:ok, map()} | {:error, term()}
  @callback update_pet(map()) :: {:ok, map()} | {:error, :not_found} | {:error, term()}
  @callback find_pets_by_status(String.t()) :: {:ok, [map()]} | {:error, term()}
  @callback find_pets_by_tags([String.t()]) :: {:ok, [map()]} | {:error, term()}
  @callback get_pet_by_id(integer()) :: {:ok, map()} | {:error, :not_found} | {:error, term()}
  @callback update_pet_with_form(integer(), String.t() | nil, String.t() | nil) ::
              {:ok, map()} | {:error, :not_found} | {:error, term()}
  @callback delete_pet(integer()) :: :ok | {:error, :not_found} | {:error, term()}
  @callback upload_file(integer(), binary()) ::
              {:ok, map()} | {:error, :not_found} | {:error, term()}

  # Store operations
  @callback get_inventory() :: {:ok, map()} | {:error, term()}
  @callback place_order(map()) :: {:ok, map()} | {:error, term()}
  @callback get_order_by_id(integer()) :: {:ok, map()} | {:error, :not_found} | {:error, term()}
  @callback delete_order(integer()) :: :ok | {:error, :not_found} | {:error, term()}

  # User operations
  @callback create_user(map()) :: {:ok, map()} | {:error, term()}
  @callback create_users_with_list([map()]) :: {:ok, [map()]} | {:error, term()}
  @callback login_user(String.t(), String.t()) :: {:ok, String.t()} | {:error, term()}
  @callback logout_user() :: :ok
  @callback get_user_by_name(String.t()) ::
              {:ok, map()} | {:error, :not_found} | {:error, term()}
  @callback update_user(String.t(), map()) ::
              {:ok, map()} | {:error, :not_found} | {:error, term()}
  @callback delete_user(String.t()) :: :ok | {:error, :not_found} | {:error, term()}
end
