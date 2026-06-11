require 'json'
require_relative 'petstore_errors'

# In-memory Petstore repository for use in tests.
# Mirrors the behaviour of PostgresPetstoreRepository without any DB connection.
class InMemoryPetstoreRepository
  def initialize
    @pets   = {}
    @orders = {}
    @users  = {}
    @photos = {}
  end

  # ---------------------------------------------------------------------------
  # Pet operations
  # ---------------------------------------------------------------------------

  def add_pet(pet)
    id = fetch(pet, :id, 'id')&.to_i || next_id(@pets)
    data = normalize_pet(pet.merge('id' => id))
    @pets[id] = data
    data
  end

  def update_pet(pet)
    id = fetch(pet, :id, 'id')&.to_i
    raise InvalidInputError, 'Pet ID required for update' if id.nil?
    raise NotFoundError, 'Pet not found' unless @pets.key?(id)

    data = normalize_pet(pet)
    @pets[id] = data
    data
  end

  def find_pet_by_id(pet_id)
    @pets.fetch(pet_id.to_i) { raise NotFoundError, 'Pet not found' }
  end

  def delete_pet(pet_id)
    @pets.delete(pet_id.to_i)
  end

  def find_pets_by_status(status)
    @pets.values.select { |p| p['status'] == status }
  end

  def find_pets_by_tags(tags)
    return [] if tags.nil? || tags.empty?

    @pets.values.select do |p|
      pet_tag_names = (p['tags'] || []).map { |t| t['name'] }
      tags.any? { |t| pet_tag_names.include?(t) }
    end
  end

  def update_pet_with_form(pet_id, name, status)
    pet = @pets.fetch(pet_id.to_i) { raise NotFoundError, 'Pet not found' }
    pet['name']   = name   if name
    pet['status'] = status if status
  end

  def upload_file(pet_id, content, content_type, metadata)
    raise NotFoundError, 'Pet not found' unless @pets.key?(pet_id.to_i)

    id = next_id(@photos)
    @photos[id] = {
      'id'          => id,
      'petId'       => pet_id.to_i,
      'content'     => content,
      'contentType' => content_type,
      'metadata'    => metadata
    }
    content.bytesize
  end

  # ---------------------------------------------------------------------------
  # Store operations
  # ---------------------------------------------------------------------------

  def get_inventory
    @pets.values.each_with_object({}) do |pet, inv|
      s = pet['status']
      next unless s
      inv[s] = (inv[s] || 0) + 1
    end
  end

  def place_order(order)
    id = fetch(order, :id, 'id')&.to_i || next_id(@orders)
    data = normalize_order(order.merge('id' => id))
    @orders[id] = data
    data
  end

  def find_order_by_id(order_id)
    @orders.fetch(order_id.to_i) { raise NotFoundError, 'Order not found' }
  end

  def delete_order(order_id)
    @orders.delete(order_id.to_i)
  end

  # ---------------------------------------------------------------------------
  # User operations
  # ---------------------------------------------------------------------------

  def create_user(user)
    username = fetch(user, :username, 'username')
    @users[username] = normalize_user(user)
    user
  end

  def create_users_with_list(users)
    users.each { |u| create_user(u) }
  end

  def find_user_by_username(username)
    @users.fetch(username) { raise NotFoundError, 'User not found' }
  end

  def update_user(username, user)
    raise NotFoundError, 'User not found' unless @users.key?(username)

    @users[username] = normalize_user(user.merge('username' => username))
  end

  def delete_user(username)
    @users.delete(username)
  end

  def login_user(username, password)
    user = @users[username]
    return false unless user

    user['password'] == password
  end

  private

  def fetch(obj, sym_key, str_key) = obj[sym_key] || obj[str_key]
  def next_id(store)                = (store.keys.max || 0) + 1

  def normalize_pet(pet)
    id = fetch(pet, :id, 'id')&.to_i
    {
      'id'        => id,
      'name'      => fetch(pet, :name, 'name'),
      'category'  => fetch(pet, :category, 'category'),
      'photoUrls' => fetch(pet, :photoUrls, 'photoUrls') || fetch(pet, :photo_urls, 'photo_urls') || [],
      'tags'      => fetch(pet, :tags, 'tags'),
      'status'    => fetch(pet, :status, 'status')
    }.compact.tap { |h| h['photoUrls'] ||= [] }
  end

  def normalize_order(order)
    id = fetch(order, :id, 'id')&.to_i
    pet_id_val = fetch(order, :petId, 'petId') || fetch(order, :pet_id, 'pet_id')
    qty = fetch(order, :quantity, 'quantity')
    {
      'id'       => id,
      'petId'    => pet_id_val&.to_i,
      'quantity' => qty&.to_i,
      'shipDate' => fetch(order, :shipDate, 'shipDate') || fetch(order, :ship_date, 'ship_date'),
      'status'   => fetch(order, :status, 'status'),
      'complete' => fetch(order, :complete, 'complete')
    }.compact
  end

  def normalize_user(user)
    id_val = fetch(user, :id, 'id')
    status_val = fetch(user, :userStatus, 'userStatus') || fetch(user, :user_status, 'user_status')
    {
      'id'         => id_val&.to_i,
      'username'   => fetch(user, :username, 'username'),
      'firstName'  => fetch(user, :firstName, 'firstName') || fetch(user, :first_name, 'first_name'),
      'lastName'   => fetch(user, :lastName, 'lastName') || fetch(user, :last_name, 'last_name'),
      'email'      => fetch(user, :email, 'email'),
      'password'   => fetch(user, :password, 'password'),
      'phone'      => fetch(user, :phone, 'phone'),
      'userStatus' => status_val&.to_i
    }.compact
  end
end
