require 'pg'
require 'json'
require_relative 'petstore_errors'

# PostgreSQL-backed implementation of the Petstore repository.
# Uses raw SQL via the pg gem — no ActiveRecord.
class PostgresPetstoreRepository
  def initialize(options = {})
    @options = {
      host:     ENV.fetch('POSTGRES_HOST', 'localhost'),
      port:     ENV.fetch('POSTGRES_PORT', '5434').to_i,
      dbname:   ENV.fetch('POSTGRES_DB', 'ruby-rails'),
      user:     ENV.fetch('POSTGRES_USER', 'myuser'),
      password: ENV.fetch('POSTGRES_PASSWORD', 'mypassword')
    }.merge(options)
  end

  # ---------------------------------------------------------------------------
  # Pet operations
  # ---------------------------------------------------------------------------

  def add_pet(pet)
    id = fetch(pet, :id, 'id')&.to_i || next_id('pet')
    exec_params(
      <<~SQL,
        INSERT INTO pet (id, name, category, photo_urls, tags, status)
        VALUES ($1, $2, $3, cast($4 as json), cast($5 as json), cast($6 as pet_status))
        ON CONFLICT (id) DO UPDATE SET
          name = EXCLUDED.name, category = EXCLUDED.category,
          photo_urls = EXCLUDED.photo_urls, tags = EXCLUDED.tags,
          status = EXCLUDED.status
      SQL
      [id, pet_name(pet), pet_category(pet),
       JSON.dump(pet_photo_urls(pet)), json_or_nil(pet_tags(pet)), pet_status(pet)]
    )
    map_pet_from_input(pet, id)
  end

  def update_pet(pet)
    id = fetch(pet, :id, 'id')&.to_i
    raise InvalidInputError, 'Pet ID required for update' if id.nil?

    result = exec_params(
      <<~SQL,
        INSERT INTO pet (id, name, category, photo_urls, tags, status)
        VALUES ($1, $2, $3, cast($4 as json), cast($5 as json), cast($6 as pet_status))
        ON CONFLICT (id) DO UPDATE SET
          name = EXCLUDED.name, category = EXCLUDED.category,
          photo_urls = EXCLUDED.photo_urls, tags = EXCLUDED.tags,
          status = EXCLUDED.status
        RETURNING id
      SQL
      [id, pet_name(pet), pet_category(pet),
       JSON.dump(pet_photo_urls(pet)), json_or_nil(pet_tags(pet)), pet_status(pet)]
    )
    raise NotFoundError, 'Pet not found' if result.ntuples.zero?
    map_pet_from_input(pet, id)
  end

  def find_pet_by_id(pet_id)
    result = exec_params(
      'SELECT id, name, category, photo_urls, tags, status::text FROM pet WHERE id = $1',
      [pet_id.to_i]
    )
    raise NotFoundError, 'Pet not found' if result.ntuples.zero?
    map_pet_row(result[0])
  end

  def delete_pet(pet_id)
    exec_params('DELETE FROM pet WHERE id = $1', [pet_id.to_i])
  end

  def find_pets_by_status(status)
    result = exec_params(
      'SELECT id, name, category, photo_urls, tags, status::text FROM pet WHERE status = cast($1 as pet_status)',
      [status]
    )
    result.map { |row| map_pet_row(row) }
  end

  def find_pets_by_tags(tags)
    return [] if tags.nil? || tags.empty?

    conditions = tags.each_with_index.map { |_, i| "tags::jsonb @> cast($#{i + 1} as jsonb)" }.join(' OR ')
    params = tags.map { |t| JSON.dump([{ 'name' => t }]) }
    result = exec_params(
      "SELECT id, name, category, photo_urls, tags, status::text FROM pet WHERE #{conditions}",
      params
    )
    result.map { |row| map_pet_row(row) }
  end

  def update_pet_with_form(pet_id, name, status)
    sets = []
    params = []
    if name
      sets << "name = $#{params.size + 1}"
      params << name
    end
    if status
      sets << "status = cast($#{params.size + 1} as pet_status)"
      params << status
    end
    return if sets.empty?

    params << pet_id.to_i
    exec_params("UPDATE pet SET #{sets.join(', ')} WHERE id = $#{params.size}", params)
  end

  def upload_file(pet_id, content, content_type, metadata)
    exists = exec_params('SELECT 1 FROM pet WHERE id = $1', [pet_id.to_i])
    raise NotFoundError, 'Pet not found' if exists.ntuples.zero?

    encoded = PG::Connection.escape_bytea(content.b)
    exec_params(
      <<~SQL,
        INSERT INTO pet_photo (id, pet_id, content_type, metadata, content)
        VALUES ((SELECT COALESCE(MAX(id), 0) + 1 FROM pet_photo), $1, $2, $3, $4::bytea)
      SQL
      [pet_id.to_i, content_type, metadata, encoded]
    )
    content.bytesize
  end

  # ---------------------------------------------------------------------------
  # Store operations
  # ---------------------------------------------------------------------------

  def get_inventory
    result = exec_params(
      "SELECT status::text, cast(COUNT(*) as int) AS cnt FROM pet GROUP BY status",
      []
    )
    result.each_with_object({}) do |row, inv|
      inv[row['status']] = row['cnt'].to_i if row['status']
    end
  end

  def place_order(order)
    id = fetch(order, :id, 'id')&.to_i || next_id('"order"')
    exec_params(
      <<~SQL,
        INSERT INTO "order" (id, pet_id, quantity, ship_date, status, complete)
        VALUES ($1, $2, $3, $4, cast($5 as order_status), $6)
        ON CONFLICT (id) DO UPDATE SET
          pet_id = EXCLUDED.pet_id, quantity = EXCLUDED.quantity,
          ship_date = EXCLUDED.ship_date, status = EXCLUDED.status,
          complete = EXCLUDED.complete
      SQL
      [id, order_pet_id(order), order_quantity(order),
       order_ship_date(order), order_status(order), order_complete(order)]
    )
    map_order_from_input(order, id)
  end

  def find_order_by_id(order_id)
    result = exec_params(
      'SELECT id, pet_id, quantity, ship_date, status::text, complete FROM "order" WHERE id = $1',
      [order_id.to_i]
    )
    raise NotFoundError, 'Order not found' if result.ntuples.zero?
    map_order_row(result[0])
  end

  def delete_order(order_id)
    exec_params('DELETE FROM "order" WHERE id = $1', [order_id.to_i])
  end

  # ---------------------------------------------------------------------------
  # User operations
  # ---------------------------------------------------------------------------

  def create_user(user)
    exec_params(
      <<~SQL,
        INSERT INTO "user" (id, username, first_name, last_name, email, password, phone, user_status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (username) DO UPDATE SET
          id = EXCLUDED.id, first_name = EXCLUDED.first_name,
          last_name = EXCLUDED.last_name, email = EXCLUDED.email,
          password = EXCLUDED.password, phone = EXCLUDED.phone,
          user_status = EXCLUDED.user_status
      SQL
      [user_id(user), user_username(user), user_first_name(user), user_last_name(user),
       user_email(user), user_password(user), user_phone(user), user_status_val(user)]
    )
    user
  end

  def create_users_with_list(users)
    users.each { |u| create_user(u) }
  end

  def find_user_by_username(username)
    result = exec_params(
      'SELECT id, username, first_name, last_name, email, password, phone, user_status FROM "user" WHERE username = $1',
      [username]
    )
    raise NotFoundError, 'User not found' if result.ntuples.zero?
    map_user_row(result[0])
  end

  def update_user(username, user)
    result = exec_params(
      <<~SQL,
        UPDATE "user" SET
          id = $1, first_name = $2, last_name = $3, email = $4,
          password = $5, phone = $6, user_status = $7
        WHERE username = $8
        RETURNING username
      SQL
      [user_id(user), user_first_name(user), user_last_name(user),
       user_email(user), user_password(user), user_phone(user),
       user_status_val(user), username]
    )
    raise NotFoundError, 'User not found' if result.ntuples.zero?
  end

  def delete_user(username)
    exec_params('DELETE FROM "user" WHERE username = $1', [username])
  end

  def login_user(username, password)
    result = exec_params(
      'SELECT 1 FROM "user" WHERE username = $1 AND password = $2',
      [username, password]
    )
    result.ntuples.positive?
  end

  private

  def conn
    Thread.current[:petstore_pg_conn] ||= PG.connect(@options)
  end

  def exec_params(sql, params)
    conn.exec_params(sql, params)
  end

  def next_id(table)
    result = exec_params("SELECT COALESCE(MAX(id), 0) + 1 AS nid FROM #{table}", [])
    result[0]['nid'].to_i
  end

  # Pet helpers
  def fetch(obj, sym_key, str_key) = obj[sym_key] || obj[str_key]
  def pet_name(pet)        = fetch(pet, :name, 'name')
  def pet_status(pet)      = fetch(pet, :status, 'status')
  def pet_photo_urls(pet)  = fetch(pet, :photoUrls, 'photoUrls') || fetch(pet, :photo_urls, 'photo_urls') || []
  def pet_tags(pet)        = fetch(pet, :tags, 'tags')

  def pet_category(pet)
    cat = fetch(pet, :category, 'category')
    cat ? JSON.dump(cat) : nil
  end

  def json_or_nil(val) = val ? JSON.dump(val) : nil

  def map_pet_from_input(pet, id)
    {
      'id'        => id,
      'name'      => pet_name(pet),
      'category'  => fetch(pet, :category, 'category'),
      'photoUrls' => pet_photo_urls(pet),
      'tags'      => pet_tags(pet),
      'status'    => pet_status(pet)
    }.compact
  end

  def map_pet_row(row)
    {
      'id'        => row['id']&.to_i,
      'name'      => row['name'],
      'category'  => row['category'] ? JSON.parse(row['category']) : nil,
      'photoUrls' => row['photo_urls'] ? JSON.parse(row['photo_urls']) : [],
      'tags'      => row['tags'] ? JSON.parse(row['tags']) : nil,
      'status'    => row['status']
    }.compact
  end

  # Order helpers
  def order_pet_id(o)    = (v = fetch(o, :petId, 'petId') || fetch(o, :pet_id, 'pet_id')) ? v.to_i : nil
  def order_quantity(o)  = (v = fetch(o, :quantity, 'quantity')) ? v.to_i : nil
  def order_status(o)    = fetch(o, :status, 'status')
  def order_complete(o)  = fetch(o, :complete, 'complete')

  def order_ship_date(order)
    v = fetch(order, :shipDate, 'shipDate') || fetch(order, :ship_date, 'ship_date')
    v ? Time.parse(v.to_s) : nil
  end

  def map_order_from_input(order, id)
    {
      'id'       => id,
      'petId'    => order_pet_id(order),
      'quantity' => order_quantity(order),
      'shipDate' => (sd = order_ship_date(order)) ? sd.iso8601 : nil,
      'status'   => order_status(order),
      'complete' => order_complete(order)
    }.compact
  end

  def map_order_row(row)
    {
      'id'       => row['id']&.to_i,
      'petId'    => row['pet_id']&.to_i,
      'quantity' => row['quantity']&.to_i,
      'shipDate' => row['ship_date'],
      'status'   => row['status'],
      'complete' => row['complete'] == 't'
    }.compact
  end

  # User helpers
  def user_id(u)         = (v = fetch(u, :id, 'id')) ? v.to_i : nil
  def user_username(u)   = fetch(u, :username, 'username')
  def user_first_name(u) = fetch(u, :firstName, 'firstName') || fetch(u, :first_name, 'first_name')
  def user_last_name(u)  = fetch(u, :lastName, 'lastName') || fetch(u, :last_name, 'last_name')
  def user_email(u)      = fetch(u, :email, 'email')
  def user_password(u)   = fetch(u, :password, 'password')
  def user_phone(u)      = fetch(u, :phone, 'phone')
  def user_status_val(u) = (v = fetch(u, :userStatus, 'userStatus') || fetch(u, :user_status, 'user_status')) ? v.to_i : nil

  def map_user_row(row)
    {
      'id'         => row['id']&.to_i,
      'username'   => row['username'],
      'firstName'  => row['first_name'],
      'lastName'   => row['last_name'],
      'email'      => row['email'],
      'password'   => row['password'],
      'phone'      => row['phone'],
      'userStatus' => row['user_status']&.to_i
    }.compact
  end
end
