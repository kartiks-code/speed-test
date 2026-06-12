defmodule PetstoreWeb.Router do
  use PetstoreWeb, :router

  pipeline :api do
    plug :accepts, ["json"]
  end

  scope "/api/v3", PetstoreWeb do
    pipe_through :api

    # Pet operations
    post "/pet", PetController, :add_pet
    put "/pet", PetController, :update_pet
    get "/pet/findByStatus", PetController, :find_by_status
    get "/pet/findByTags", PetController, :find_by_tags
    get "/pet/:petId", PetController, :get_pet_by_id
    post "/pet/:petId", PetController, :update_with_form
    delete "/pet/:petId", PetController, :delete_pet
    post "/pet/:petId/uploadImage", PetController, :upload_file

    # Store operations
    get "/store/inventory", StoreController, :get_inventory
    post "/store/order", StoreController, :place_order
    get "/store/order/:orderId", StoreController, :get_order_by_id
    delete "/store/order/:orderId", StoreController, :delete_order

    # User operations
    post "/user", UserController, :create_user
    post "/user/createWithList", UserController, :create_with_list
    get "/user/login", UserController, :login
    get "/user/logout", UserController, :logout
    get "/user/:username", UserController, :get_by_name
    put "/user/:username", UserController, :update_user
    delete "/user/:username", UserController, :delete_user
  end
end
