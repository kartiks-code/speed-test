defmodule Petstore.InMemoryRepository do
  @moduledoc """
  In-memory fake repository backed by an Agent. Used in tests.
  All state is reset when the process restarts.
  """
  @behaviour Petstore.Repository
  use Agent

  def start_link(_opts \\ []) do
    Agent.start_link(fn -> initial_state() end, name: __MODULE__)
  end

  @doc "Resets all in-memory state. Use in test setup instead of restarting."
  def reset do
    Agent.update(__MODULE__, fn _ -> initial_state() end)
  end

  defp initial_state, do: %{pets: %{}, orders: %{}, users: %{}}

  defp state, do: Agent.get(__MODULE__, & &1)
  defp update_state(fun), do: Agent.update(__MODULE__, fun)

  # ---------------------------------------------------------------------------
  # Pet
  # ---------------------------------------------------------------------------

  @impl true
  def add_pet(params) do
    id = params["id"] || params[:id]
    pets = state().pets

    id =
      if id && id != 0 do
        id
      else
        case Enum.max_by(Map.keys(pets), & &1, fn -> 0 end) do
          0 -> 1
          max -> max + 1
        end
      end

    pet = normalize_pet(Map.merge(stringify_keys(params), %{"id" => id}))
    update_state(fn s -> put_in(s[:pets][id], pet) end)
    {:ok, pet}
  end

  @impl true
  def update_pet(params) do
    id = params["id"] || params[:id]

    case state().pets[id] do
      nil -> {:error, :not_found}
      _existing ->
        pet = normalize_pet(Map.merge(state().pets[id], stringify_keys(params)))
        update_state(fn s -> put_in(s[:pets][id], pet) end)
        {:ok, pet}
    end
  end

  @impl true
  def find_pets_by_status(status) do
    pets =
      state().pets
      |> Map.values()
      |> Enum.filter(fn p -> p["status"] == status end)

    {:ok, pets}
  end

  @impl true
  def find_pets_by_tags(tags) when is_list(tags) do
    pets =
      state().pets
      |> Map.values()
      |> Enum.filter(fn p ->
        pet_tag_names = Enum.map(p["tags"] || [], fn t -> t["name"] end)
        Enum.any?(tags, fn tag -> tag in pet_tag_names end)
      end)

    {:ok, pets}
  end

  @impl true
  def get_pet_by_id(id) do
    case state().pets[id] do
      nil -> {:error, :not_found}
      pet -> {:ok, pet}
    end
  end

  @impl true
  def update_pet_with_form(id, name, status) do
    case state().pets[id] do
      nil ->
        {:error, :not_found}

      existing ->
        updated =
          existing
          |> maybe_put("name", name)
          |> maybe_put("status", status)
          |> normalize_pet()

        update_state(fn s -> put_in(s[:pets][id], updated) end)
        {:ok, updated}
    end
  end

  @impl true
  def delete_pet(id) do
    case state().pets[id] do
      nil -> {:error, :not_found}
      _ ->
        update_state(fn s -> update_in(s[:pets], &Map.delete(&1, id)) end)
        :ok
    end
  end

  @impl true
  def upload_file(pet_id, data) when is_binary(data) do
    case state().pets[pet_id] do
      nil -> {:error, :not_found}
      _ -> {:ok, %{"code" => 200, "type" => "application/octet-stream", "message" => "Uploaded #{byte_size(data)} bytes"}}
    end
  end

  # ---------------------------------------------------------------------------
  # Store
  # ---------------------------------------------------------------------------

  @impl true
  def get_inventory do
    inventory =
      state().pets
      |> Map.values()
      |> Enum.group_by(fn p -> p["status"] end)
      |> Enum.into(%{}, fn {k, v} -> {k, length(v)} end)

    {:ok, inventory}
  end

  @impl true
  def place_order(params) do
    id = params["id"] || params[:id]
    orders = state().orders

    id =
      if id && id != 0 do
        id
      else
        case Enum.max_by(Map.keys(orders), & &1, fn -> 0 end) do
          0 -> 1
          max -> max + 1
        end
      end

    order = normalize_order(Map.merge(stringify_keys(params), %{"id" => id}))
    update_state(fn s -> put_in(s[:orders][id], order) end)
    {:ok, order}
  end

  @impl true
  def get_order_by_id(id) do
    case state().orders[id] do
      nil -> {:error, :not_found}
      order -> {:ok, order}
    end
  end

  @impl true
  def delete_order(id) do
    case state().orders[id] do
      nil -> {:error, :not_found}
      _ ->
        update_state(fn s -> update_in(s[:orders], &Map.delete(&1, id)) end)
        :ok
    end
  end

  # ---------------------------------------------------------------------------
  # User
  # ---------------------------------------------------------------------------

  @impl true
  def create_user(params) do
    user = normalize_user(stringify_keys(params))
    username = user["username"]
    update_state(fn s -> put_in(s[:users][username], user) end)
    {:ok, user}
  end

  @impl true
  def create_users_with_list(users) when is_list(users) do
    results = Enum.map(users, fn u ->
      {:ok, user} = create_user(u)
      user
    end)
    {:ok, results}
  end

  @impl true
  def login_user(_username, _password) do
    {:ok, "logged-in-session-token"}
  end

  @impl true
  def logout_user, do: :ok

  @impl true
  def get_user_by_name(username) do
    case state().users[username] do
      nil -> {:error, :not_found}
      user -> {:ok, user}
    end
  end

  @impl true
  def update_user(username, params) do
    case state().users[username] do
      nil -> {:error, :not_found}
      existing ->
        user = normalize_user(Map.merge(existing, stringify_keys(params)))
        update_state(fn s -> put_in(s[:users][username], user) end)
        {:ok, user}
    end
  end

  @impl true
  def delete_user(username) do
    case state().users[username] do
      nil -> {:error, :not_found}
      _ ->
        update_state(fn s -> update_in(s[:users], &Map.delete(&1, username)) end)
        :ok
    end
  end

  # ---------------------------------------------------------------------------
  # Private helpers
  # ---------------------------------------------------------------------------

  defp normalize_pet(p) do
    %{
      "id" => p["id"],
      "name" => p["name"] || "",
      "status" => p["status"] || "available",
      "category" => p["category"],
      "photoUrls" => p["photoUrls"] || [],
      "tags" => p["tags"] || []
    }
  end

  defp normalize_order(o) do
    %{
      "id" => o["id"],
      "petId" => o["petId"] || o["pet_id"],
      "quantity" => o["quantity"] || 0,
      "shipDate" => o["shipDate"] || o["ship_date"],
      "status" => o["status"] || "placed",
      "complete" => o["complete"] || false
    }
  end

  defp normalize_user(u) do
    %{
      "id" => u["id"],
      "username" => u["username"] || "",
      "firstName" => u["firstName"] || u["first_name"] || "",
      "lastName" => u["lastName"] || u["last_name"] || "",
      "email" => u["email"] || "",
      "password" => u["password"] || "",
      "phone" => u["phone"] || "",
      "userStatus" => u["userStatus"] || u["user_status"] || 0
    }
  end

  defp stringify_keys(map) when is_map(map) do
    Map.new(map, fn
      {k, v} when is_atom(k) -> {Atom.to_string(k), v}
      {k, v} -> {k, v}
    end)
  end

  defp maybe_put(map, _key, nil), do: map
  defp maybe_put(map, key, value), do: Map.put(map, key, value)
end
