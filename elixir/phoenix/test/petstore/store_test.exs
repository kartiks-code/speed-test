defmodule Petstore.StoreTest do
  use ExUnit.Case, async: false

  alias Petstore.InMemoryRepository, as: Repo

  setup do
    {:ok, _} = start_supervised(Repo)
    :ok
  end

  describe "place_order/1" do
    test "creates an order with provided id" do
      {:ok, order} = Repo.place_order(%{"id" => 1, "petId" => 10, "quantity" => 2, "status" => "placed"})
      assert order["id"] == 1
      assert order["petId"] == 10
      assert order["quantity"] == 2
      assert order["status"] == "placed"
    end

    test "assigns ID when not provided" do
      {:ok, order} = Repo.place_order(%{"petId" => 5, "quantity" => 1})
      assert order["id"] != nil
      assert order["id"] > 0
    end

    test "assigns sequential IDs" do
      {:ok, o1} = Repo.place_order(%{"petId" => 1})
      {:ok, o2} = Repo.place_order(%{"petId" => 2})
      assert o2["id"] == o1["id"] + 1
    end

    test "defaults status to placed" do
      {:ok, order} = Repo.place_order(%{"petId" => 1})
      assert order["status"] == "placed"
    end

    test "defaults complete to false" do
      {:ok, order} = Repo.place_order(%{"petId" => 1})
      assert order["complete"] == false
    end
  end

  describe "get_order_by_id/1" do
    test "retrieves existing order" do
      {:ok, _} = Repo.place_order(%{"id" => 100, "petId" => 5, "quantity" => 3})
      {:ok, order} = Repo.get_order_by_id(100)
      assert order["petId"] == 5
      assert order["quantity"] == 3
    end

    test "returns not_found for missing order" do
      assert {:error, :not_found} = Repo.get_order_by_id(9999)
    end
  end

  describe "delete_order/1" do
    test "deletes existing order" do
      Repo.place_order(%{"id" => 50, "petId" => 1})
      assert :ok = Repo.delete_order(50)
      assert {:error, :not_found} = Repo.get_order_by_id(50)
    end

    test "returns not_found for missing order" do
      assert {:error, :not_found} = Repo.delete_order(9999)
    end
  end

  describe "get_inventory/0" do
    test "returns empty map when no pets" do
      {:ok, inventory} = Repo.get_inventory()
      assert inventory == %{}
    end

    test "counts pets by status" do
      Repo.add_pet(%{"id" => 1, "name" => "A", "status" => "available"})
      Repo.add_pet(%{"id" => 2, "name" => "B", "status" => "available"})
      Repo.add_pet(%{"id" => 3, "name" => "C", "status" => "sold"})

      {:ok, inventory} = Repo.get_inventory()
      assert inventory["available"] == 2
      assert inventory["sold"] == 1
    end
  end
end
