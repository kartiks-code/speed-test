Rails.application.routes.draw do
  scope '/api/v3' do
    # Pet — static paths before dynamic :petId
    post   'pet',                      to: 'pet#create'
    put    'pet',                      to: 'pet#update_pet'
    get    'pet/findByStatus',         to: 'pet#find_pets_by_status'
    get    'pet/findByTags',           to: 'pet#find_pets_by_tags'
    get    'pet/:petId',               to: 'pet#show'
    post   'pet/:petId',               to: 'pet#update_pet_with_form'
    delete 'pet/:petId',               to: 'pet#destroy'
    post   'pet/:petId/uploadImage',   to: 'pet#upload_file'

    # Store
    get    'store/inventory',          to: 'store#get_inventory'
    post   'store/order',              to: 'store#place_order'
    get    'store/order/:orderId',     to: 'store#get_order_by_id'
    delete 'store/order/:orderId',     to: 'store#delete_order'

    # User — static paths before dynamic :username
    post   'user',                     to: 'user#create'
    post   'user/createWithList',      to: 'user#create_users_with_list_input'
    get    'user/login',               to: 'user#login_user'
    get    'user/logout',              to: 'user#logout_user'
    get    'user/:username',           to: 'user#show'
    put    'user/:username',           to: 'user#update'
    delete 'user/:username',           to: 'user#destroy'
  end
end
