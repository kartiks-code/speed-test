defmodule PetstoreWeb.PetController do
  use PetstoreWeb, :controller
  import PetstoreWeb.RepoHelper

  # POST /pet
  def add_pet(conn, params) do
    case repo().add_pet(params) do
      {:ok, pet} -> json(conn, pet)
      {:error, reason} -> send_error(conn, 400, reason)
    end
  end

  # PUT /pet
  def update_pet(conn, params) do
    case repo().update_pet(params) do
      {:ok, pet} -> json(conn, pet)
      {:error, :not_found} -> send_error(conn, 404, "Pet not found")
      {:error, reason} -> send_error(conn, 400, reason)
    end
  end

  # GET /pet/findByStatus?status=available
  def find_by_status(conn, %{"status" => status}) do
    case repo().find_pets_by_status(status) do
      {:ok, pets} -> json(conn, pets)
      {:error, reason} -> send_error(conn, 400, reason)
    end
  end

  def find_by_status(conn, _params) do
    send_error(conn, 400, "Missing status parameter")
  end

  # GET /pet/findByTags?tags=tag1,tag2
  def find_by_tags(conn, params) do
    tags =
      case params["tags"] do
        nil -> []
        t when is_list(t) -> t
        t when is_binary(t) -> String.split(t, ",", trim: true)
      end

    case repo().find_pets_by_tags(tags) do
      {:ok, pets} -> json(conn, pets)
      {:error, reason} -> send_error(conn, 400, reason)
    end
  end

  # GET /pet/:petId
  def get_pet_by_id(conn, %{"petId" => pet_id}) do
    case Integer.parse(pet_id) do
      {id, ""} ->
        case repo().get_pet_by_id(id) do
          {:ok, pet} -> json(conn, pet)
          {:error, :not_found} -> send_error(conn, 404, "Pet not found")
          {:error, reason} -> send_error(conn, 400, reason)
        end

      _ ->
        send_error(conn, 400, "Invalid pet ID")
    end
  end

  # POST /pet/:petId  (update with form)
  def update_with_form(conn, %{"petId" => pet_id} = params) do
    case Integer.parse(pet_id) do
      {id, ""} ->
        name = params["name"]
        status = params["status"]

        case repo().update_pet_with_form(id, name, status) do
          {:ok, pet} -> json(conn, pet)
          {:error, :not_found} -> send_error(conn, 404, "Pet not found")
          {:error, reason} -> send_error(conn, 400, reason)
        end

      _ ->
        send_error(conn, 400, "Invalid pet ID")
    end
  end

  # DELETE /pet/:petId
  def delete_pet(conn, %{"petId" => pet_id}) do
    case Integer.parse(pet_id) do
      {id, ""} ->
        case repo().delete_pet(id) do
          :ok -> send_resp(conn, 200, "")
          {:error, :not_found} -> send_error(conn, 404, "Pet not found")
          {:error, reason} -> send_error(conn, 400, reason)
        end

      _ ->
        send_error(conn, 400, "Invalid pet ID")
    end
  end

  # POST /pet/:petId/uploadImage
  def upload_file(conn, %{"petId" => pet_id}) do
    case Integer.parse(pet_id) do
      {id, ""} ->
        body = conn.assigns[:raw_body] || read_body(conn)
        data = extract_body(body)

        case repo().upload_file(id, data) do
          {:ok, response} -> json(conn, response)
          {:error, :not_found} -> send_error(conn, 404, "Pet not found")
          {:error, reason} -> send_error(conn, 400, reason)
        end

      _ ->
        send_error(conn, 400, "Invalid pet ID")
    end
  end

  defp extract_body({:ok, data, _conn}), do: data
  defp extract_body(data) when is_binary(data), do: data
  defp extract_body(_), do: ""

  defp send_error(conn, status, message) when is_binary(message) do
    conn
    |> put_status(status)
    |> json(%{"message" => message})
  end

  defp send_error(conn, status, reason) do
    send_error(conn, status, inspect(reason))
  end
end
