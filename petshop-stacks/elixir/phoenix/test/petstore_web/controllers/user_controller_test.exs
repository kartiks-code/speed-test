defmodule PetstoreWeb.UserControllerTest do
  use PetstoreWeb.ConnCase, async: false

  @base "/api/v3"

  describe "POST /user" do
    test "creates a user and returns 200", %{conn: conn} do
      conn = post(conn, "#{@base}/user", %{
        "username" => "testuser",
        "firstName" => "Test",
        "lastName" => "User",
        "email" => "test@example.com",
        "password" => "secret",
        "phone" => "555-1234",
        "userStatus" => 1
      })
      body = json_response(conn, 200)
      assert body["username"] == "testuser"
      assert body["firstName"] == "Test"
      assert body["email"] == "test@example.com"
    end

    test "defaults missing fields to empty strings", %{conn: conn} do
      conn = post(conn, "#{@base}/user", %{"username" => "minimal"})
      body = json_response(conn, 200)
      assert body["firstName"] == ""
      assert body["userStatus"] == 0
    end
  end

  describe "POST /user/createWithList" do
    test "creates multiple users from a JSON array", %{conn: conn} do
      body = Jason.encode!([%{"username" => "user1"}, %{"username" => "user2"}])

      conn =
        conn
        |> put_req_header("content-type", "application/json")
        |> post("#{@base}/user/createWithList", body)

      users = json_response(conn, 200)
      assert length(users) == 2
      usernames = Enum.map(users, & &1["username"])
      assert "user1" in usernames
      assert "user2" in usernames
    end

    test "returns empty list for empty JSON array", %{conn: conn} do
      body = Jason.encode!([])

      conn =
        conn
        |> put_req_header("content-type", "application/json")
        |> post("#{@base}/user/createWithList", body)

      assert json_response(conn, 200) == []
    end

    test "returns empty list for non-list body", %{conn: conn} do
      conn = post(conn, "#{@base}/user/createWithList", %{"not" => "a list"})
      assert json_response(conn, 200) == []
    end
  end

  describe "GET /user/login" do
    test "returns a session token with rate-limit headers", %{conn: conn} do
      conn = get(conn, "#{@base}/user/login", username: "anyone", password: "pass")
      assert conn.status == 200
      assert get_resp_header(conn, "x-rate-limit") == ["5000"]
      assert get_resp_header(conn, "x-expires-after") != []
    end

    test "returns 400 when username is missing", %{conn: conn} do
      conn = get(conn, "#{@base}/user/login", password: "pass")
      assert json_response(conn, 400)["message"] =~ "Missing"
    end

    test "returns 400 when password is missing", %{conn: conn} do
      conn = get(conn, "#{@base}/user/login", username: "foo")
      assert json_response(conn, 400)["message"] =~ "Missing"
    end
  end

  describe "GET /user/logout" do
    test "returns 200 (stateless no-op)", %{conn: conn} do
      conn = get(conn, "#{@base}/user/logout")
      assert conn.status == 200
    end
  end

  describe "GET /user/:username" do
    test "returns existing user by username", %{conn: conn} do
      post(conn, "#{@base}/user", %{"username" => "alice", "email" => "alice@example.com"})
      conn = get(build_conn(), "#{@base}/user/alice")
      body = json_response(conn, 200)
      assert body["email"] == "alice@example.com"
    end

    test "returns 404 for missing user", %{conn: conn} do
      conn = get(conn, "#{@base}/user/nobody")
      assert json_response(conn, 404)["message"] =~ "not found"
    end
  end

  describe "PUT /user/:username" do
    test "updates existing user fields", %{conn: conn} do
      post(conn, "#{@base}/user", %{"username" => "bob", "email" => "old@example.com"})
      conn = put(build_conn(), "#{@base}/user/bob", %{"email" => "new@example.com"})
      body = json_response(conn, 200)
      assert body["email"] == "new@example.com"
    end

    test "returns 404 for missing user", %{conn: conn} do
      conn = put(conn, "#{@base}/user/ghost", %{"email" => "x@y.com"})
      assert json_response(conn, 404)["message"] =~ "not found"
    end
  end

  describe "DELETE /user/:username" do
    test "deletes existing user and returns 200", %{conn: conn} do
      post(conn, "#{@base}/user", %{"username" => "toDelete"})
      conn = delete(build_conn(), "#{@base}/user/toDelete")
      assert conn.status == 200
    end

    test "returns 404 for missing user", %{conn: conn} do
      conn = delete(conn, "#{@base}/user/nobody")
      assert json_response(conn, 404)["message"] =~ "not found"
    end
  end
end
