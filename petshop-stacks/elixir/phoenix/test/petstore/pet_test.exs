defmodule Petstore.PetTest do
  use ExUnit.Case, async: false

  alias Petstore.InMemoryRepository, as: Repo

  setup do
    Repo.reset()
    :ok
  end

  describe "add_pet/1" do
    test "creates a pet with provided id" do
      {:ok, pet} = Repo.add_pet(%{"id" => 1, "name" => "Doggo", "status" => "available"})
      assert pet["id"] == 1
      assert pet["name"] == "Doggo"
      assert pet["status"] == "available"
    end

    test "assigns ID when not provided" do
      {:ok, pet} = Repo.add_pet(%{"name" => "Kitty", "status" => "pending"})
      assert pet["id"] != nil
      assert pet["id"] > 0
    end

    test "assigns sequential IDs" do
      {:ok, p1} = Repo.add_pet(%{"name" => "Pet1", "status" => "available"})
      {:ok, p2} = Repo.add_pet(%{"name" => "Pet2", "status" => "available"})
      assert p2["id"] == p1["id"] + 1
    end

    test "defaults status to available when not provided" do
      {:ok, pet} = Repo.add_pet(%{"name" => "NoStatus"})
      assert pet["status"] == "available"
    end

    test "sets empty photoUrls by default" do
      {:ok, pet} = Repo.add_pet(%{"name" => "NoPics"})
      assert pet["photoUrls"] == []
    end

    test "sets empty tags by default" do
      {:ok, pet} = Repo.add_pet(%{"name" => "NoTags"})
      assert pet["tags"] == []
    end
  end

  describe "get_pet_by_id/1" do
    test "retrieves existing pet" do
      {:ok, _} = Repo.add_pet(%{"id" => 42, "name" => "Rex", "status" => "available"})
      {:ok, pet} = Repo.get_pet_by_id(42)
      assert pet["name"] == "Rex"
    end

    test "returns not_found for missing pet" do
      assert {:error, :not_found} = Repo.get_pet_by_id(9999)
    end
  end

  describe "update_pet/1" do
    test "updates an existing pet" do
      {:ok, _} = Repo.add_pet(%{"id" => 1, "name" => "OldName", "status" => "available"})
      {:ok, updated} = Repo.update_pet(%{"id" => 1, "name" => "NewName", "status" => "sold"})
      assert updated["name"] == "NewName"
      assert updated["status"] == "sold"
    end

    test "returns not_found when pet doesn't exist" do
      assert {:error, :not_found} = Repo.update_pet(%{"id" => 9999, "name" => "Ghost"})
    end
  end

  describe "find_pets_by_status/1" do
    test "finds pets with matching status" do
      Repo.add_pet(%{"id" => 1, "name" => "A", "status" => "available"})
      Repo.add_pet(%{"id" => 2, "name" => "B", "status" => "sold"})
      Repo.add_pet(%{"id" => 3, "name" => "C", "status" => "available"})

      {:ok, pets} = Repo.find_pets_by_status("available")
      assert length(pets) == 2
      assert Enum.all?(pets, fn p -> p["status"] == "available" end)
    end

    test "returns empty list when no matches" do
      {:ok, pets} = Repo.find_pets_by_status("sold")
      assert pets == []
    end
  end

  describe "find_pets_by_tags/1" do
    test "finds pets with matching tag" do
      Repo.add_pet(%{"id" => 1, "name" => "A", "tags" => [%{"name" => "cute"}]})
      Repo.add_pet(%{"id" => 2, "name" => "B", "tags" => [%{"name" => "wild"}]})

      {:ok, pets} = Repo.find_pets_by_tags(["cute"])
      assert length(pets) == 1
      assert hd(pets)["name"] == "A"
    end

    test "returns empty list with no tag matches" do
      {:ok, pets} = Repo.find_pets_by_tags(["nonexistent"])
      assert pets == []
    end
  end

  describe "update_pet_with_form/3" do
    test "updates name and status" do
      Repo.add_pet(%{"id" => 5, "name" => "Old", "status" => "available"})
      {:ok, pet} = Repo.update_pet_with_form(5, "New", "sold")
      assert pet["name"] == "New"
      assert pet["status"] == "sold"
    end

    test "ignores nil name" do
      Repo.add_pet(%{"id" => 5, "name" => "Keep", "status" => "available"})
      {:ok, pet} = Repo.update_pet_with_form(5, nil, "sold")
      assert pet["name"] == "Keep"
    end

    test "returns not_found for missing pet" do
      assert {:error, :not_found} = Repo.update_pet_with_form(9999, "Name", "available")
    end
  end

  describe "delete_pet/1" do
    test "deletes existing pet" do
      Repo.add_pet(%{"id" => 10, "name" => "ToDelete", "status" => "available"})
      assert :ok = Repo.delete_pet(10)
      assert {:error, :not_found} = Repo.get_pet_by_id(10)
    end

    test "returns not_found for missing pet" do
      assert {:error, :not_found} = Repo.delete_pet(9999)
    end
  end

  describe "upload_file/2" do
    test "uploads file for existing pet" do
      Repo.add_pet(%{"id" => 1, "name" => "Test", "status" => "available"})
      {:ok, response} = Repo.upload_file(1, "binary data")
      assert response["code"] == 200
      assert response["message"] =~ "11 bytes"
    end

    test "returns not_found for missing pet" do
      assert {:error, :not_found} = Repo.upload_file(9999, "data")
    end

    test "reports correct byte count for empty binary" do
      Repo.add_pet(%{"id" => 1, "name" => "Test", "status" => "available"})
      {:ok, response} = Repo.upload_file(1, "")
      assert response["message"] =~ "0 bytes"
    end
  end

  describe "add_pet/1 with atom keys" do
    test "accepts atom keys and converts them" do
      {:ok, pet} = Repo.add_pet(%{id: 55, name: "AtomKeyed", status: "available"})
      assert pet["id"] == 55
      assert pet["name"] == "AtomKeyed"
    end
  end

  describe "add_pet/1 with id = 0" do
    test "assigns a new ID when id is 0" do
      {:ok, pet} = Repo.add_pet(%{"id" => 0, "name" => "ZeroId"})
      assert pet["id"] != 0
      assert pet["id"] >= 1
    end
  end

  describe "find_pets_by_tags/1 with multiple tags" do
    test "returns pets matching any of the tags" do
      Repo.add_pet(%{"id" => 1, "name" => "A", "tags" => [%{"name" => "cute"}]})
      Repo.add_pet(%{"id" => 2, "name" => "B", "tags" => [%{"name" => "wild"}]})
      Repo.add_pet(%{"id" => 3, "name" => "C", "tags" => [%{"name" => "other"}]})

      {:ok, pets} = Repo.find_pets_by_tags(["cute", "wild"])
      assert length(pets) == 2
    end
  end

  describe "update_pet_with_form/3 ignores nil status" do
    test "preserves existing status when status is nil" do
      Repo.add_pet(%{"id" => 1, "name" => "Test", "status" => "available"})
      {:ok, pet} = Repo.update_pet_with_form(1, "New", nil)
      assert pet["status"] == "available"
    end
  end

  describe "normalize_pet preserves category" do
    test "stores and retrieves category field" do
      {:ok, pet} = Repo.add_pet(%{
        "id" => 1,
        "name" => "Fluffy",
        "status" => "available",
        "category" => %{"id" => 1, "name" => "Dogs"}
      })
      assert pet["category"] == %{"id" => 1, "name" => "Dogs"}
    end
  end
end
