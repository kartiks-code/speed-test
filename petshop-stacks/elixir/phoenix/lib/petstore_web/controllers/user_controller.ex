defmodule PetstoreWeb.UserController do
  use PetstoreWeb, :controller
  import PetstoreWeb.RepoHelper

  # POST /user
  def create_user(conn, params) do
    case repo().create_user(params) do
      {:ok, user} -> json(conn, user)
      {:error, reason} -> send_error(conn, 400, reason)
    end
  end

  # POST /user/createWithList
  def create_with_list(conn, params) do
    users =
      case params do
        %{"_json" => list} when is_list(list) -> list
        list when is_list(list) -> list
        _ -> []
      end

    case repo().create_users_with_list(users) do
      {:ok, result} -> json(conn, result)
      {:error, reason} -> send_error(conn, 400, reason)
    end
  end

  # GET /user/login?username=x&password=y
  def login(conn, %{"username" => username, "password" => password}) do
    case repo().login_user(username, password) do
      {:ok, token} ->
        conn
        |> put_resp_header("x-rate-limit", "5000")
        |> put_resp_header("x-expires-after", "2099-01-01T00:00:00Z")
        |> json(token)

      {:error, reason} ->
        send_error(conn, 400, reason)
    end
  end

  def login(conn, _params) do
    send_error(conn, 400, "Missing username or password")
  end

  # GET /user/logout
  def logout(conn, _params) do
    repo().logout_user()
    send_resp(conn, 200, "")
  end

  # GET /user/:username
  def get_by_name(conn, %{"username" => username}) do
    case repo().get_user_by_name(username) do
      {:ok, user} -> json(conn, user)
      {:error, :not_found} -> send_error(conn, 404, "User not found")
      {:error, reason} -> send_error(conn, 400, reason)
    end
  end

  # PUT /user/:username
  def update_user(conn, %{"username" => username} = params) do
    update_params = Map.delete(params, "username")

    case repo().update_user(username, update_params) do
      {:ok, user} -> json(conn, user)
      {:error, :not_found} -> send_error(conn, 404, "User not found")
      {:error, reason} -> send_error(conn, 400, reason)
    end
  end

  # DELETE /user/:username
  def delete_user(conn, %{"username" => username}) do
    case repo().delete_user(username) do
      :ok -> send_resp(conn, 200, "")
      {:error, :not_found} -> send_error(conn, 404, "User not found")
      {:error, reason} -> send_error(conn, 400, reason)
    end
  end

  defp send_error(conn, status, message) when is_binary(message) do
    conn
    |> put_status(status)
    |> json(%{"message" => message})
  end

  defp send_error(conn, status, reason) do
    send_error(conn, status, inspect(reason))
  end
end
