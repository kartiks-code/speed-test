defmodule PetstoreWeb.StoreController do
  use PetstoreWeb, :controller
  import PetstoreWeb.RepoHelper

  # GET /store/inventory
  def get_inventory(conn, _params) do
    case repo().get_inventory() do
      {:ok, inventory} -> json(conn, inventory)
      {:error, reason} -> send_error(conn, 500, reason)
    end
  end

  # POST /store/order
  def place_order(conn, params) do
    case repo().place_order(params) do
      {:ok, order} -> json(conn, order)
      {:error, reason} -> send_error(conn, 400, reason)
    end
  end

  # GET /store/order/:orderId
  def get_order_by_id(conn, %{"orderId" => order_id}) do
    case Integer.parse(order_id) do
      {id, ""} ->
        case repo().get_order_by_id(id) do
          {:ok, order} -> json(conn, order)
          {:error, :not_found} -> send_error(conn, 404, "Order not found")
          {:error, reason} -> send_error(conn, 400, reason)
        end

      _ ->
        send_error(conn, 400, "Invalid order ID")
    end
  end

  # DELETE /store/order/:orderId
  def delete_order(conn, %{"orderId" => order_id}) do
    case Integer.parse(order_id) do
      {id, ""} ->
        case repo().delete_order(id) do
          :ok -> send_resp(conn, 200, "")
          {:error, :not_found} -> send_error(conn, 404, "Order not found")
          {:error, reason} -> send_error(conn, 400, reason)
        end

      _ ->
        send_error(conn, 400, "Invalid order ID")
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
