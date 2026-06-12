defmodule Petstore.UserTest do
  use ExUnit.Case, async: false

  alias Petstore.InMemoryRepository, as: Repo

  setup do
    {:ok, _} = start_supervised(Repo)
    :ok
  end

  describe "create_user/1" do
    test "creates a user with string keys" do
      {:ok, user} = Repo.create_user(%{
        "username" => "testuser",
        "firstName" => "Test",
        "lastName" => "User",
        "email" => "test@example.com",
        "password" => "secret",
        "phone" => "555-1234",
        "userStatus" => 1
      })
      assert user["username"] == "testuser"
      assert user["firstName"] == "Test"
      assert user["email"] == "test@example.com"
    end

    test "defaults missing fields to empty strings" do
      {:ok, user} = Repo.create_user(%{"username" => "minimal"})
      assert user["firstName"] == ""
      assert user["lastName"] == ""
      assert user["email"] == ""
    end

    test "defaults userStatus to 0" do
      {:ok, user} = Repo.create_user(%{"username" => "zero_status"})
      assert user["userStatus"] == 0
    end
  end

  describe "create_users_with_list/1" do
    test "creates multiple users" do
      {:ok, users} = Repo.create_users_with_list([
        %{"username" => "user1"},
        %{"username" => "user2"}
      ])
      assert length(users) == 2
    end
  end

  describe "get_user_by_name/1" do
    test "retrieves existing user" do
      Repo.create_user(%{"username" => "alice", "email" => "alice@example.com"})
      {:ok, user} = Repo.get_user_by_name("alice")
      assert user["email"] == "alice@example.com"
    end

    test "returns not_found for missing user" do
      assert {:error, :not_found} = Repo.get_user_by_name("nobody")
    end
  end

  describe "update_user/2" do
    test "updates existing user fields" do
      Repo.create_user(%{"username" => "bob", "email" => "old@example.com"})
      {:ok, updated} = Repo.update_user("bob", %{"email" => "new@example.com"})
      assert updated["email"] == "new@example.com"
    end

    test "preserves unchanged fields" do
      Repo.create_user(%{"username" => "carol", "firstName" => "Carol", "email" => "carol@example.com"})
      {:ok, updated} = Repo.update_user("carol", %{"email" => "newemail@example.com"})
      assert updated["firstName"] == "Carol"
    end

    test "returns not_found for missing user" do
      assert {:error, :not_found} = Repo.update_user("ghost", %{"email" => "x@y.com"})
    end
  end

  describe "delete_user/1" do
    test "deletes existing user" do
      Repo.create_user(%{"username" => "toDelete"})
      assert :ok = Repo.delete_user("toDelete")
      assert {:error, :not_found} = Repo.get_user_by_name("toDelete")
    end

    test "returns not_found for missing user" do
      assert {:error, :not_found} = Repo.delete_user("nobody")
    end
  end

  describe "login_user/2" do
    test "returns a session token" do
      {:ok, token} = Repo.login_user("anyone", "password")
      assert is_binary(token)
      assert String.length(token) > 0
    end
  end

  describe "logout_user/0" do
    test "returns :ok (stateless)" do
      assert :ok = Repo.logout_user()
    end
  end
end
