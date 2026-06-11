defmodule Petstore.PostgresRepository do
  @moduledoc "Postgrex-backed implementation of Petstore.Repository."
  @behaviour Petstore.Repository

  defp db, do: Petstore.DB

  # ---------------------------------------------------------------------------
  # Pet
  # ---------------------------------------------------------------------------

  @impl true
  def add_pet(params) do
    id = params["id"] || params[:id]

    id =
      if id && id != 0 do
        id
      else
        case Postgrex.query(db(), "SELECT COALESCE(MAX(id), 0) + 1 AS next_id FROM pet", []) do
          {:ok, %{rows: [[next_id]]}} -> next_id
          _ -> 1
        end
      end

    pet = Map.merge(params, %{"id" => id})
    upsert_pet(pet)
  end

  @impl true
  def update_pet(params) do
    id = params["id"] || params[:id]

    case get_pet_by_id(id) do
      {:error, :not_found} -> {:error, :not_found}
      {:ok, _} -> upsert_pet(params)
      err -> err
    end
  end

  defp upsert_pet(pet) do
    id = pet["id"] || pet[:id]
    name = pet["name"] || pet[:name] || ""
    status = pet["status"] || pet[:status] || "available"
    category = Jason.encode!(pet["category"] || pet[:category])
    photo_urls = Jason.encode!(pet["photoUrls"] || pet[:photo_urls] || [])
    tags = Jason.encode!(pet["tags"] || pet[:tags] || [])

    sql = """
    INSERT INTO pet (id, name, status, category, photo_urls, tags)
    VALUES ($1, $2, $3::pet_status, $4, $5, $6)
    ON CONFLICT (id) DO UPDATE SET
      name = EXCLUDED.name,
      status = EXCLUDED.status,
      category = EXCLUDED.category,
      photo_urls = EXCLUDED.photo_urls,
      tags = EXCLUDED.tags
    RETURNING id, name, status::text, category, photo_urls, tags
    """

    case Postgrex.query(db(), sql, [id, name, status, category, photo_urls, tags]) do
      {:ok, %{rows: [row], columns: cols}} -> {:ok, row_to_pet(cols, row)}
      {:error, reason} -> {:error, reason}
    end
  end

  @impl true
  def find_pets_by_status(status) do
    sql = "SELECT id, name, status::text, category, photo_urls, tags FROM pet WHERE status = $1::pet_status"

    case Postgrex.query(db(), sql, [status]) do
      {:ok, %{rows: rows, columns: cols}} -> {:ok, Enum.map(rows, &row_to_pet(cols, &1))}
      {:error, reason} -> {:error, reason}
    end
  end

  @impl true
  def find_pets_by_tags(tags) when is_list(tags) do
    conditions =
      tags
      |> Enum.with_index(1)
      |> Enum.map(fn {_tag, i} -> "tags::jsonb @> $#{i}::jsonb" end)
      |> Enum.join(" OR ")

    sql =
      "SELECT id, name, status::text, category, photo_urls, tags FROM pet WHERE #{conditions}"

    params = Enum.map(tags, fn tag -> Jason.encode!([%{"name" => tag}]) end)

    case Postgrex.query(db(), sql, params) do
      {:ok, %{rows: rows, columns: cols}} -> {:ok, Enum.map(rows, &row_to_pet(cols, &1))}
      {:error, reason} -> {:error, reason}
    end
  end

  @impl true
  def get_pet_by_id(id) do
    sql = "SELECT id, name, status::text, category, photo_urls, tags FROM pet WHERE id = $1"

    case Postgrex.query(db(), sql, [id]) do
      {:ok, %{rows: [], columns: _}} -> {:error, :not_found}
      {:ok, %{rows: [row], columns: cols}} -> {:ok, row_to_pet(cols, row)}
      {:error, reason} -> {:error, reason}
    end
  end

  @impl true
  def update_pet_with_form(id, name, status) do
    case get_pet_by_id(id) do
      {:error, :not_found} ->
        {:error, :not_found}

      {:ok, existing} ->
        updated =
          existing
          |> maybe_put("name", name)
          |> maybe_put("status", status)

        upsert_pet(updated)

      err ->
        err
    end
  end

  @impl true
  def delete_pet(id) do
    case get_pet_by_id(id) do
      {:error, :not_found} ->
        {:error, :not_found}

      {:ok, _} ->
        case Postgrex.query(db(), "DELETE FROM pet WHERE id = $1", [id]) do
          {:ok, _} -> :ok
          {:error, reason} -> {:error, reason}
        end

      err ->
        err
    end
  end

  @impl true
  def upload_file(pet_id, data) when is_binary(data) do
    case get_pet_by_id(pet_id) do
      {:error, :not_found} ->
        {:error, :not_found}

      {:ok, _} ->
        sql = """
        INSERT INTO pet_photo (pet_id, content)
        VALUES ($1, $2)
        ON CONFLICT (pet_id) DO UPDATE SET content = EXCLUDED.content
        """

        case Postgrex.query(db(), sql, [pet_id, data]) do
          {:ok, _} -> {:ok, %{"code" => 200, "type" => "application/octet-stream", "message" => "Uploaded #{byte_size(data)} bytes"}}
          {:error, reason} -> {:error, reason}
        end

      err ->
        err
    end
  end

  # ---------------------------------------------------------------------------
  # Store
  # ---------------------------------------------------------------------------

  @impl true
  def get_inventory do
    sql = "SELECT status::text, COUNT(*) FROM pet GROUP BY status"

    case Postgrex.query(db(), sql, []) do
      {:ok, %{rows: rows}} ->
        inventory = Enum.reduce(rows, %{}, fn [status, count], acc -> Map.put(acc, status, count) end)
        {:ok, inventory}

      {:error, reason} ->
        {:error, reason}
    end
  end

  @impl true
  def place_order(params) do
    id = params["id"] || params[:id]

    id =
      if id && id != 0 do
        id
      else
        case Postgrex.query(db(), ~s(SELECT COALESCE(MAX(id), 0) + 1 AS next_id FROM "order"), []) do
          {:ok, %{rows: [[next_id]]}} -> next_id
          _ -> 1
        end
      end

    order = Map.merge(params, %{"id" => id})
    upsert_order(order)
  end

  defp upsert_order(order) do
    id = order["id"] || order[:id]
    pet_id = order["petId"] || order[:pet_id]
    quantity = order["quantity"] || order[:quantity] || 0
    ship_date = order["shipDate"] || order[:ship_date]
    status = order["status"] || order[:status] || "placed"
    complete = order["complete"] || order[:complete] || false

    sql = """
    INSERT INTO "order" (id, pet_id, quantity, ship_date, status, complete)
    VALUES ($1, $2, $3, $4::timestamptz, $5::order_status, $6)
    ON CONFLICT (id) DO UPDATE SET
      pet_id = EXCLUDED.pet_id,
      quantity = EXCLUDED.quantity,
      ship_date = EXCLUDED.ship_date,
      status = EXCLUDED.status,
      complete = EXCLUDED.complete
    RETURNING id, pet_id, quantity, ship_date, status::text, complete
    """

    case Postgrex.query(db(), sql, [id, pet_id, quantity, ship_date, status, complete]) do
      {:ok, %{rows: [row], columns: cols}} -> {:ok, row_to_order(cols, row)}
      {:error, reason} -> {:error, reason}
    end
  end

  @impl true
  def get_order_by_id(id) do
    sql = ~s(SELECT id, pet_id, quantity, ship_date, status::text, complete FROM "order" WHERE id = $1)

    case Postgrex.query(db(), sql, [id]) do
      {:ok, %{rows: [], columns: _}} -> {:error, :not_found}
      {:ok, %{rows: [row], columns: cols}} -> {:ok, row_to_order(cols, row)}
      {:error, reason} -> {:error, reason}
    end
  end

  @impl true
  def delete_order(id) do
    case get_order_by_id(id) do
      {:error, :not_found} ->
        {:error, :not_found}

      {:ok, _} ->
        case Postgrex.query(db(), ~s(DELETE FROM "order" WHERE id = $1), [id]) do
          {:ok, _} -> :ok
          {:error, reason} -> {:error, reason}
        end

      err ->
        err
    end
  end

  # ---------------------------------------------------------------------------
  # User
  # ---------------------------------------------------------------------------

  @impl true
  def create_user(params) do
    upsert_user(params)
  end

  @impl true
  def create_users_with_list(users) when is_list(users) do
    results =
      Enum.map(users, fn user ->
        case create_user(user) do
          {:ok, u} -> u
          _ -> nil
        end
      end)
      |> Enum.reject(&is_nil/1)

    {:ok, results}
  end

  defp upsert_user(user) do
    username = user["username"] || user[:username] || ""
    first_name = user["firstName"] || user[:first_name] || ""
    last_name = user["lastName"] || user[:last_name] || ""
    email = user["email"] || user[:email] || ""
    password = user["password"] || user[:password] || ""
    phone = user["phone"] || user[:phone] || ""
    user_status = user["userStatus"] || user[:user_status] || 0

    sql = """
    INSERT INTO "user" (username, first_name, last_name, email, password, phone, user_status)
    VALUES ($1, $2, $3, $4, $5, $6, $7)
    ON CONFLICT (username) DO UPDATE SET
      first_name = EXCLUDED.first_name,
      last_name = EXCLUDED.last_name,
      email = EXCLUDED.email,
      password = EXCLUDED.password,
      phone = EXCLUDED.phone,
      user_status = EXCLUDED.user_status
    RETURNING id, username, first_name, last_name, email, password, phone, user_status
    """

    case Postgrex.query(db(), sql, [username, first_name, last_name, email, password, phone, user_status]) do
      {:ok, %{rows: [row], columns: cols}} -> {:ok, row_to_user(cols, row)}
      {:error, reason} -> {:error, reason}
    end
  end

  @impl true
  def login_user(_username, _password) do
    {:ok, "logged-in-session-token"}
  end

  @impl true
  def logout_user, do: :ok

  @impl true
  def get_user_by_name(username) do
    sql = ~s(SELECT id, username, first_name, last_name, email, password, phone, user_status FROM "user" WHERE username = $1)

    case Postgrex.query(db(), sql, [username]) do
      {:ok, %{rows: [], columns: _}} -> {:error, :not_found}
      {:ok, %{rows: [row], columns: cols}} -> {:ok, row_to_user(cols, row)}
      {:error, reason} -> {:error, reason}
    end
  end

  @impl true
  def update_user(username, params) do
    case get_user_by_name(username) do
      {:error, :not_found} -> {:error, :not_found}
      {:ok, existing} -> upsert_user(Map.merge(existing, params))
      err -> err
    end
  end

  @impl true
  def delete_user(username) do
    case get_user_by_name(username) do
      {:error, :not_found} ->
        {:error, :not_found}

      {:ok, _} ->
        case Postgrex.query(db(), ~s(DELETE FROM "user" WHERE username = $1), [username]) do
          {:ok, _} -> :ok
          {:error, reason} -> {:error, reason}
        end

      err ->
        err
    end
  end

  # ---------------------------------------------------------------------------
  # Private helpers
  # ---------------------------------------------------------------------------

  defp row_to_pet(cols, row) do
    cols
    |> Enum.zip(row)
    |> Enum.into(%{})
    |> decode_json_fields(["category", "photo_urls", "tags"])
    |> rename_pet_keys()
  end

  defp rename_pet_keys(m) do
    %{
      "id" => m["id"],
      "name" => m["name"],
      "status" => m["status"],
      "category" => m["category"],
      "photoUrls" => m["photo_urls"] || [],
      "tags" => m["tags"] || []
    }
  end

  defp row_to_order(cols, row) do
    cols
    |> Enum.zip(row)
    |> Enum.into(%{})
    |> rename_order_keys()
  end

  defp rename_order_keys(m) do
    ship_date =
      case m["ship_date"] do
        nil -> nil
        %DateTime{} = dt -> DateTime.to_iso8601(dt)
        %NaiveDateTime{} = ndt -> NaiveDateTime.to_iso8601(ndt) <> "Z"
        s when is_binary(s) -> s
      end

    %{
      "id" => m["id"],
      "petId" => m["pet_id"],
      "quantity" => m["quantity"],
      "shipDate" => ship_date,
      "status" => m["status"],
      "complete" => m["complete"] || false
    }
  end

  defp row_to_user(cols, row) do
    cols
    |> Enum.zip(row)
    |> Enum.into(%{})
    |> rename_user_keys()
  end

  defp rename_user_keys(m) do
    %{
      "id" => m["id"],
      "username" => m["username"],
      "firstName" => m["first_name"],
      "lastName" => m["last_name"],
      "email" => m["email"],
      "password" => m["password"],
      "phone" => m["phone"],
      "userStatus" => m["user_status"]
    }
  end

  defp decode_json_fields(map, fields) do
    Enum.reduce(fields, map, fn field, acc ->
      case acc[field] do
        nil -> acc
        v when is_binary(v) -> Map.put(acc, field, Jason.decode!(v))
        _ -> acc
      end
    end)
  end

  defp maybe_put(map, _key, nil), do: map
  defp maybe_put(map, key, value), do: Map.put(map, key, value)
end
