defmodule PetstoreWeb.ErrorJSON do
  def render("404.json", _assigns) do
    %{message: "Not found"}
  end

  def render("400.json", _assigns) do
    %{message: "Bad request"}
  end

  def render("405.json", _assigns) do
    %{message: "Method not allowed"}
  end

  def render(template, _assigns) do
    %{message: Phoenix.Controller.status_message_from_template(template)}
  end
end
