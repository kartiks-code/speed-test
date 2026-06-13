defmodule PetstoreWeb.PetControllerTest do
  use PetstoreWeb.ConnCase, async: false

  @base "/api/v3"

  describe "POST /pet" do
    test "creates a pet and returns 200 with the pet", %{conn: conn} do
      conn = post(conn, "#{@base}/pet", %{"name" => "Doggo", "status" => "available"})
      body = json_response(conn, 200)
      assert body["name"] == "Doggo"
      assert body["status"] == "available"
      assert is_integer(body["id"])
    end

    test "assigns sequential IDs for new pets", %{conn: conn} do
      conn1 = post(conn, "#{@base}/pet", %{"name" => "First", "status" => "available"})
      conn2 = post(build_conn(), "#{@base}/pet", %{"name" => "Second", "status" => "available"})
      id1 = json_response(conn1, 200)["id"]
      id2 = json_response(conn2, 200)["id"]
      assert id2 == id1 + 1
    end
  end

  describe "PUT /pet" do
    test "updates an existing pet", %{conn: conn} do
      post(conn, "#{@base}/pet", %{"id" => 1, "name" => "OldName", "status" => "available"})
      conn = put(build_conn(), "#{@base}/pet", %{"id" => 1, "name" => "NewName", "status" => "sold"})
      body = json_response(conn, 200)
      assert body["name"] == "NewName"
      assert body["status"] == "sold"
    end

    test "returns 404 when pet not found", %{conn: conn} do
      conn = put(conn, "#{@base}/pet", %{"id" => 9999, "name" => "Ghost"})
      assert json_response(conn, 404)["message"] =~ "not found"
    end
  end

  describe "GET /pet/findByStatus" do
    test "returns pets matching the status", %{conn: conn} do
      post(conn, "#{@base}/pet", %{"name" => "A", "status" => "available"})
      post(build_conn(), "#{@base}/pet", %{"name" => "B", "status" => "sold"})
      conn = get(build_conn(), "#{@base}/pet/findByStatus", status: "available")
      pets = json_response(conn, 200)
      assert length(pets) == 1
      assert hd(pets)["name"] == "A"
    end

    test "returns 400 when status param is missing", %{conn: conn} do
      conn = get(conn, "#{@base}/pet/findByStatus")
      assert json_response(conn, 400)["message"] =~ "Missing status"
    end
  end

  describe "GET /pet/findByTags" do
    test "returns pets matching tags (comma-separated string)", %{conn: conn} do
      post(conn, "#{@base}/pet", %{"name" => "TaggedPet", "tags" => [%{"name" => "cute"}]})
      conn = get(build_conn(), "#{@base}/pet/findByTags", tags: "cute")
      pets = json_response(conn, 200)
      assert length(pets) == 1
      assert hd(pets)["name"] == "TaggedPet"
    end

    test "returns pets matching tags (list)", %{conn: conn} do
      post(conn, "#{@base}/pet", %{"name" => "TaggedPet", "tags" => [%{"name" => "wild"}]})
      conn = get(build_conn(), "#{@base}/pet/findByTags", tags: ["wild"])
      pets = json_response(conn, 200)
      assert length(pets) == 1
    end

    test "returns empty list when tags param is missing", %{conn: conn} do
      post(conn, "#{@base}/pet", %{"name" => "NoPet"})
      conn = get(build_conn(), "#{@base}/pet/findByTags")
      assert json_response(conn, 200) == []
    end
  end

  describe "GET /pet/:petId" do
    test "returns existing pet by ID", %{conn: conn} do
      post(conn, "#{@base}/pet", %{"id" => 42, "name" => "Rex", "status" => "available"})
      conn = get(build_conn(), "#{@base}/pet/42")
      body = json_response(conn, 200)
      assert body["name"] == "Rex"
    end

    test "returns 404 for missing pet", %{conn: conn} do
      conn = get(conn, "#{@base}/pet/9999")
      assert json_response(conn, 404)["message"] =~ "not found"
    end

    test "returns 400 for non-integer pet ID", %{conn: conn} do
      conn = get(conn, "#{@base}/pet/abc")
      assert json_response(conn, 400)["message"] =~ "Invalid pet ID"
    end
  end

  describe "POST /pet/:petId (update with form)" do
    test "updates pet name and status", %{conn: conn} do
      post(conn, "#{@base}/pet", %{"id" => 5, "name" => "Old", "status" => "available"})
      conn = post(build_conn(), "#{@base}/pet/5", %{"name" => "New", "status" => "sold"})
      body = json_response(conn, 200)
      assert body["name"] == "New"
      assert body["status"] == "sold"
    end

    test "returns 404 for missing pet", %{conn: conn} do
      conn = post(conn, "#{@base}/pet/9999", %{"name" => "Ghost"})
      assert json_response(conn, 404)["message"] =~ "not found"
    end

    test "returns 400 for invalid pet ID", %{conn: conn} do
      conn = post(conn, "#{@base}/pet/abc", %{"name" => "X"})
      assert json_response(conn, 400)["message"] =~ "Invalid pet ID"
    end
  end

  describe "DELETE /pet/:petId" do
    test "deletes an existing pet and returns 200", %{conn: conn} do
      post(conn, "#{@base}/pet", %{"id" => 10, "name" => "ToDelete"})
      conn = delete(build_conn(), "#{@base}/pet/10")
      assert conn.status == 200
    end

    test "returns 404 for missing pet", %{conn: conn} do
      conn = delete(conn, "#{@base}/pet/9999")
      assert json_response(conn, 404)["message"] =~ "not found"
    end

    test "returns 400 for invalid pet ID", %{conn: conn} do
      conn = delete(conn, "#{@base}/pet/notanumber")
      assert json_response(conn, 400)["message"] =~ "Invalid pet ID"
    end
  end

  describe "POST /pet/:petId/uploadImage" do
    test "uses raw_body from conn assigns when pre-set", %{conn: conn} do
      post(conn, "#{@base}/pet", %{"id" => 1, "name" => "Test"})

      conn =
        build_conn()
        |> assign(:raw_body, "pre-read binary")
        |> put_req_header("content-type", "application/octet-stream")
        |> post("#{@base}/pet/1/uploadImage", "")

      body = json_response(conn, 200)
      assert body["code"] == 200
      assert body["message"] =~ "bytes"
    end

    test "uploads binary data for existing pet", %{conn: conn} do
      post(conn, "#{@base}/pet", %{"id" => 1, "name" => "Test"})

      conn =
        build_conn()
        |> put_req_header("content-type", "application/octet-stream")
        |> post("#{@base}/pet/1/uploadImage", "binary data here")

      body = json_response(conn, 200)
      assert body["code"] == 200
      assert body["message"] =~ "bytes"
    end

    test "returns 404 for missing pet", %{conn: _conn} do
      conn =
        build_conn()
        |> put_req_header("content-type", "application/octet-stream")
        |> post("#{@base}/pet/9999/uploadImage", "data")

      assert json_response(conn, 404)["message"] =~ "not found"
    end

    test "returns 400 for invalid pet ID", %{conn: conn} do
      conn =
        conn
        |> put_req_header("content-type", "application/octet-stream")
        |> post("#{@base}/pet/bad/uploadImage", "data")

      assert json_response(conn, 400)["message"] =~ "Invalid pet ID"
    end
  end
end
