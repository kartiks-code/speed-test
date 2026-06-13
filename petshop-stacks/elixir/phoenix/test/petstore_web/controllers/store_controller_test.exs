defmodule PetstoreWeb.StoreControllerTest do
  use PetstoreWeb.ConnCase, async: false

  @base "/api/v3"

  describe "GET /store/inventory" do
    test "returns empty inventory when no pets", %{conn: conn} do
      conn = get(conn, "#{@base}/store/inventory")
      assert json_response(conn, 200) == %{}
    end

    test "returns pet counts by status", %{conn: conn} do
      post(conn, "#{@base}/pet", %{"name" => "A", "status" => "available"})
      post(build_conn(), "#{@base}/pet", %{"name" => "B", "status" => "available"})
      post(build_conn(), "#{@base}/pet", %{"name" => "C", "status" => "sold"})

      conn = get(build_conn(), "#{@base}/store/inventory")
      body = json_response(conn, 200)
      assert body["available"] == 2
      assert body["sold"] == 1
    end
  end

  describe "POST /store/order" do
    test "creates an order and returns 200", %{conn: conn} do
      conn = post(conn, "#{@base}/store/order", %{
        "petId" => 5,
        "quantity" => 2,
        "status" => "placed"
      })
      body = json_response(conn, 200)
      assert body["petId"] == 5
      assert body["quantity"] == 2
      assert body["status"] == "placed"
      assert is_integer(body["id"])
    end

    test "defaults status to placed and complete to false", %{conn: conn} do
      conn = post(conn, "#{@base}/store/order", %{"petId" => 1})
      body = json_response(conn, 200)
      assert body["status"] == "placed"
      assert body["complete"] == false
    end
  end

  describe "GET /store/order/:orderId" do
    test "returns existing order by ID", %{conn: conn} do
      post(conn, "#{@base}/store/order", %{"id" => 100, "petId" => 5, "quantity" => 3})
      conn = get(build_conn(), "#{@base}/store/order/100")
      body = json_response(conn, 200)
      assert body["petId"] == 5
      assert body["quantity"] == 3
    end

    test "returns 404 for missing order", %{conn: conn} do
      conn = get(conn, "#{@base}/store/order/9999")
      assert json_response(conn, 404)["message"] =~ "not found"
    end

    test "returns 400 for non-integer order ID", %{conn: conn} do
      conn = get(conn, "#{@base}/store/order/abc")
      assert json_response(conn, 400)["message"] =~ "Invalid order ID"
    end
  end

  describe "DELETE /store/order/:orderId" do
    test "deletes an existing order and returns 200", %{conn: conn} do
      post(conn, "#{@base}/store/order", %{"id" => 50, "petId" => 1})
      conn = delete(build_conn(), "#{@base}/store/order/50")
      assert conn.status == 200
    end

    test "returns 404 for missing order", %{conn: conn} do
      conn = delete(conn, "#{@base}/store/order/9999")
      assert json_response(conn, 404)["message"] =~ "not found"
    end

    test "returns 400 for invalid order ID", %{conn: conn} do
      conn = delete(conn, "#{@base}/store/order/notanumber")
      assert json_response(conn, 400)["message"] =~ "Invalid order ID"
    end
  end
end
