defmodule PetstoreWeb.ErrorJSONTest do
  use ExUnit.Case, async: true

  alias PetstoreWeb.ErrorJSON

  test "renders 404" do
    assert ErrorJSON.render("404.json", %{}) == %{message: "Not found"}
  end

  test "renders 400" do
    assert ErrorJSON.render("400.json", %{}) == %{message: "Bad request"}
  end

  test "renders 405" do
    assert ErrorJSON.render("405.json", %{}) == %{message: "Method not allowed"}
  end

  test "renders any other status via status_message_from_template" do
    result = ErrorJSON.render("500.json", %{})
    assert is_map(result)
    assert Map.has_key?(result, :message)
    assert is_binary(result.message)
  end
end
