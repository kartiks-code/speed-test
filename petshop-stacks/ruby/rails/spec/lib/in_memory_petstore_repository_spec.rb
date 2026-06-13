require 'spec_helper'

RSpec.describe InMemoryPetstoreRepository do
  subject(:repo) { described_class.new }

  let(:pet) do
    {
      'name'      => 'Fido',
      'photoUrls' => ['http://example.com/fido.jpg'],
      'tags'      => [{ 'id' => 1, 'name' => 'dog' }],
      'status'    => 'available'
    }
  end

  let(:cat_pet) do
    {
      'name'      => 'Whiskers',
      'photoUrls' => [],
      'tags'      => [{ 'id' => 2, 'name' => 'cat' }],
      'status'    => 'pending'
    }
  end

  # ---------------------------------------------------------------------------
  # Pet CRUD
  # ---------------------------------------------------------------------------

  describe '#add_pet' do
    it 'assigns a new id when none is provided' do
      result = repo.add_pet(pet)
      expect(result['id']).to be_a(Integer)
    end

    it 'uses a provided id' do
      result = repo.add_pet(pet.merge('id' => 42))
      expect(result['id']).to eq(42)
    end

    it 'persists the pet' do
      result = repo.add_pet(pet)
      fetched = repo.find_pet_by_id(result['id'])
      expect(fetched['name']).to eq('Fido')
    end

    it 'upserts when the same id is added again' do
      repo.add_pet(pet.merge('id' => 10, 'name' => 'Old'))
      repo.add_pet(pet.merge('id' => 10, 'name' => 'New'))
      expect(repo.find_pet_by_id(10)['name']).to eq('New')
    end

    it 'increments id for each new pet' do
      id1 = repo.add_pet(pet)['id']
      id2 = repo.add_pet(pet)['id']
      expect(id2).to be > id1
    end

    it 'preserves all fields in the returned pet' do
      full_pet = {
        'id'        => 7,
        'name'      => 'Buddy',
        'category'  => { 'id' => 1, 'name' => 'dogs' },
        'photoUrls' => ['http://example.com/photo.jpg'],
        'tags'      => [{ 'id' => 1, 'name' => 'labrador' }],
        'status'    => 'available'
      }
      result = repo.add_pet(full_pet)
      expect(result['id']).to eq(7)
      expect(result['name']).to eq('Buddy')
      expect(result['category']).to eq({ 'id' => 1, 'name' => 'dogs' })
      expect(result['photoUrls']).to eq(['http://example.com/photo.jpg'])
      expect(result['tags']).to eq([{ 'id' => 1, 'name' => 'labrador' }])
      expect(result['status']).to eq('available')
    end

    it 'accepts symbol keys for all pet fields' do
      sym_pet = {
        id:        8,
        name:      'SymPet',
        category:  { 'id' => 2, 'name' => 'cats' },
        photoUrls: ['http://example.com/sym.jpg'],
        tags:      [{ 'id' => 3, 'name' => 'tabby' }],
        status:    'pending'
      }
      result = repo.add_pet(sym_pet)
      expect(result['id']).to eq(8)
      expect(result['name']).to eq('SymPet')
      expect(result['category']).to eq({ 'id' => 2, 'name' => 'cats' })
      expect(result['photoUrls']).to eq(['http://example.com/sym.jpg'])
      expect(result['tags']).to eq([{ 'id' => 3, 'name' => 'tabby' }])
      expect(result['status']).to eq('pending')
    end

    it 'accepts snake_case photo_urls key' do
      result = repo.add_pet({ 'name' => 'SnakePet', 'photo_urls' => ['http://example.com/snake.jpg'], 'status' => 'available' })
      expect(result['photoUrls']).to eq(['http://example.com/snake.jpg'])
    end

    it 'defaults photoUrls to empty array when absent' do
      result = repo.add_pet({ 'name' => 'NoPics', 'status' => 'available' })
      expect(result['photoUrls']).to eq([])
    end

    it 'uses max existing id plus one for auto-assigned id' do
      repo.add_pet(pet.merge('id' => 10))
      repo.add_pet(pet.merge('id' => 3))
      id = repo.add_pet(pet)['id']
      expect(id).to eq(11)
    end
  end

  describe '#update_pet' do
    it 'updates an existing pet' do
      added = repo.add_pet(pet)
      repo.update_pet(added.merge('name' => 'Rex'))
      expect(repo.find_pet_by_id(added['id'])['name']).to eq('Rex')
    end

    it 'raises NotFoundError when pet does not exist' do
      expect { repo.update_pet(pet.merge('id' => 9999)) }.to raise_error(NotFoundError)
    end

    it 'raises InvalidInputError when no id is provided' do
      expect { repo.update_pet(pet) }.to raise_error(InvalidInputError)
    end

    it 'returns the full normalized pet with all fields updated' do
      repo.add_pet(pet.merge('id' => 5))
      result = repo.update_pet({
        'id'        => 5,
        'name'      => 'Rex',
        'category'  => { 'id' => 2, 'name' => 'guard dogs' },
        'photoUrls' => ['http://example.com/rex.jpg'],
        'tags'      => [{ 'id' => 5, 'name' => 'german-shepherd' }],
        'status'    => 'sold'
      })
      expect(result['id']).to eq(5)
      expect(result['name']).to eq('Rex')
      expect(result['category']).to eq({ 'id' => 2, 'name' => 'guard dogs' })
      expect(result['photoUrls']).to eq(['http://example.com/rex.jpg'])
      expect(result['tags']).to eq([{ 'id' => 5, 'name' => 'german-shepherd' }])
      expect(result['status']).to eq('sold')
    end

    it 'converts string id to integer for lookup' do
      repo.add_pet(pet.merge('id' => 5))
      result = repo.update_pet(pet.merge('id' => '5', 'name' => 'StringId'))
      expect(result['id']).to eq(5)
      expect(result['name']).to eq('StringId')
    end
  end

  describe '#find_pet_by_id' do
    it 'returns the pet for a known id' do
      added = repo.add_pet(pet)
      expect(repo.find_pet_by_id(added['id'])['name']).to eq('Fido')
    end

    it 'raises NotFoundError for unknown id' do
      expect { repo.find_pet_by_id(9999) }.to raise_error(NotFoundError)
    end

    it 'converts string id to integer for lookup' do
      repo.add_pet(pet.merge('id' => 5))
      expect(repo.find_pet_by_id('5')['name']).to eq('Fido')
    end
  end

  describe '#delete_pet' do
    it 'removes the pet' do
      added = repo.add_pet(pet)
      repo.delete_pet(added['id'])
      expect { repo.find_pet_by_id(added['id']) }.to raise_error(NotFoundError)
    end

    it 'does not raise when pet does not exist' do
      expect { repo.delete_pet(9999) }.not_to raise_error
    end

    it 'converts string id to integer for deletion' do
      repo.add_pet(pet.merge('id' => 5))
      repo.delete_pet('5')
      expect { repo.find_pet_by_id(5) }.to raise_error(NotFoundError)
    end
  end

  describe '#find_pets_by_status' do
    before do
      repo.add_pet(pet)
      repo.add_pet(cat_pet)
    end

    it 'returns pets matching the status' do
      result = repo.find_pets_by_status('available')
      expect(result.map { |p| p['name'] }).to contain_exactly('Fido')
    end

    it 'returns an empty array when no pets match' do
      expect(repo.find_pets_by_status('sold')).to be_empty
    end

    it 'does not raise for pets without a status field' do
      repo.add_pet({ 'name' => 'Nameless', 'photoUrls' => [] })
      expect { repo.find_pets_by_status('available') }.not_to raise_error
    end

    it 'excludes pets that have no status field' do
      repo.add_pet({ 'name' => 'Nameless', 'photoUrls' => [] })
      result = repo.find_pets_by_status('available')
      expect(result.map { |p| p['name'] }).to contain_exactly('Fido')
    end
  end

  describe '#find_pets_by_tags' do
    before do
      repo.add_pet(pet)
      repo.add_pet(cat_pet)
    end

    it 'returns pets whose tags include any of the requested names' do
      result = repo.find_pets_by_tags(['dog'])
      expect(result.map { |p| p['name'] }).to contain_exactly('Fido')
    end

    it 'returns multiple matches when multiple tags match' do
      result = repo.find_pets_by_tags(['dog', 'cat'])
      expect(result.size).to eq(2)
    end

    it 'returns empty when no tags match' do
      expect(repo.find_pets_by_tags(['fish'])).to be_empty
    end

    it 'returns empty for empty tag list' do
      expect(repo.find_pets_by_tags([])).to eq([])
    end

    it 'returns empty for nil tags' do
      expect(repo.find_pets_by_tags(nil)).to eq([])
    end

    it 'does not match a pet that has no tags at all' do
      repo.add_pet({ 'name' => 'Tagless', 'photoUrls' => [], 'status' => 'available' })
      result = repo.find_pets_by_tags(['dog'])
      expect(result.map { |p| p['name'] }).to contain_exactly('Fido')
    end

    it 'matches only pets whose tags include ANY of the requested names, not ALL' do
      result = repo.find_pets_by_tags(['dog', 'fish'])
      expect(result.map { |p| p['name'] }).to contain_exactly('Fido')
    end
  end

  describe '#update_pet_with_form' do
    it 'updates the name' do
      added = repo.add_pet(pet)
      repo.update_pet_with_form(added['id'], 'Buddy', nil)
      expect(repo.find_pet_by_id(added['id'])['name']).to eq('Buddy')
    end

    it 'updates the status' do
      added = repo.add_pet(pet)
      repo.update_pet_with_form(added['id'], nil, 'sold')
      expect(repo.find_pet_by_id(added['id'])['status']).to eq('sold')
    end

    it 'raises NotFoundError for unknown id' do
      expect { repo.update_pet_with_form(9999, 'x', nil) }.to raise_error(NotFoundError)
    end

    it 'does not update name when name is nil' do
      added = repo.add_pet(pet)
      repo.update_pet_with_form(added['id'], nil, 'sold')
      expect(repo.find_pet_by_id(added['id'])['name']).to eq('Fido')
    end

    it 'does not update status when status is nil' do
      added = repo.add_pet(pet)
      repo.update_pet_with_form(added['id'], 'Buddy', nil)
      expect(repo.find_pet_by_id(added['id'])['status']).to eq('available')
    end

    it 'converts string pet_id to integer' do
      added = repo.add_pet(pet)
      repo.update_pet_with_form(added['id'].to_s, 'StringId', nil)
      expect(repo.find_pet_by_id(added['id'])['name']).to eq('StringId')
    end
  end

  describe '#upload_file' do
    it 'returns the byte count of the uploaded content' do
      added = repo.add_pet(pet)
      content = 'hello world'
      size = repo.upload_file(added['id'], content, 'image/jpeg', nil)
      expect(size).to eq(content.bytesize)
    end

    it 'raises NotFoundError when pet does not exist' do
      expect { repo.upload_file(9999, 'data', 'image/jpeg', nil) }.to raise_error(NotFoundError)
    end

    it 'returns exact byte count for ASCII content' do
      added = repo.add_pet(pet)
      content = 'abc'
      size = repo.upload_file(added['id'], content, 'image/png', nil)
      expect(size).to eq(3)
    end

    it 'returns exact byte count for binary-like content' do
      added = repo.add_pet(pet)
      content = 'x' * 1024
      size = repo.upload_file(added['id'], content, 'image/jpeg', 'meta')
      expect(size).to eq(1024)
    end

    it 'converts string pet_id to integer for existence check' do
      repo.add_pet(pet.merge('id' => 5))
      expect { repo.upload_file('5', 'data', 'image/jpeg', nil) }.not_to raise_error
    end
  end

  # ---------------------------------------------------------------------------
  # Store operations
  # ---------------------------------------------------------------------------

  describe '#get_inventory' do
    it 'returns counts grouped by status' do
      repo.add_pet(pet)
      repo.add_pet(pet.merge('name' => 'Rex'))
      repo.add_pet(cat_pet)
      inv = repo.get_inventory
      expect(inv['available']).to eq(2)
      expect(inv['pending']).to eq(1)
    end

    it 'returns an empty hash when there are no pets' do
      expect(repo.get_inventory).to eq({})
    end

    it 'does not raise for pets without a status field' do
      repo.add_pet({ 'name' => 'Nostatus', 'photoUrls' => [] })
      expect { repo.get_inventory }.not_to raise_error
    end

    it 'excludes pets without a status from counts' do
      repo.add_pet(pet)
      repo.add_pet({ 'name' => 'Nostatus', 'photoUrls' => [] })
      inv = repo.get_inventory
      expect(inv['available']).to eq(1)
      expect(inv.keys).to contain_exactly('available')
    end

    it 'counts each status independently' do
      repo.add_pet(pet.merge('status' => 'available'))
      repo.add_pet(pet.merge('status' => 'available'))
      repo.add_pet(pet.merge('status' => 'sold'))
      inv = repo.get_inventory
      expect(inv['available']).to eq(2)
      expect(inv['sold']).to eq(1)
    end
  end

  describe '#place_order' do
    let(:order) { { 'petId' => 1, 'quantity' => 2, 'status' => 'placed' } }

    it 'assigns a new id' do
      result = repo.place_order(order)
      expect(result['id']).to be_a(Integer)
    end

    it 'persists the order' do
      result = repo.place_order(order)
      fetched = repo.find_order_by_id(result['id'])
      expect(fetched['quantity']).to eq(2)
    end

    it 'uses provided id' do
      result = repo.place_order(order.merge('id' => 99))
      expect(result['id']).to eq(99)
    end

    it 'preserves all order fields in the returned order' do
      full_order = {
        'id'       => 5,
        'petId'    => 10,
        'quantity' => 3,
        'shipDate' => '2024-06-01T00:00:00Z',
        'status'   => 'approved',
        'complete' => true
      }
      result = repo.place_order(full_order)
      expect(result['id']).to eq(5)
      expect(result['petId']).to eq(10)
      expect(result['quantity']).to eq(3)
      expect(result['shipDate']).to eq('2024-06-01T00:00:00Z')
      expect(result['status']).to eq('approved')
      expect(result['complete']).to eq(true)
    end

    it 'accepts snake_case pet_id key' do
      result = repo.place_order({ 'pet_id' => 7, 'quantity' => 1, 'status' => 'placed' })
      expect(result['petId']).to eq(7)
    end

    it 'accepts snake_case ship_date key' do
      result = repo.place_order({ 'petId' => 1, 'quantity' => 1, 'ship_date' => '2024-01-01', 'status' => 'placed' })
      expect(result['shipDate']).to eq('2024-01-01')
    end

    it 'converts string quantity to integer' do
      result = repo.place_order({ 'petId' => 1, 'quantity' => '5', 'status' => 'placed' })
      expect(result['quantity']).to eq(5)
    end

    it 'converts string petId to integer' do
      result = repo.place_order({ 'petId' => '3', 'quantity' => 1, 'status' => 'placed' })
      expect(result['petId']).to eq(3)
    end

    it 'accepts symbol keys for order fields' do
      result = repo.place_order({ petId: 4, quantity: 2, status: 'placed', id: 20 })
      expect(result['id']).to eq(20)
      expect(result['petId']).to eq(4)
      expect(result['quantity']).to eq(2)
      expect(result['status']).to eq('placed')
    end
  end

  describe '#find_order_by_id' do
    it 'raises NotFoundError for unknown order' do
      expect { repo.find_order_by_id(9999) }.to raise_error(NotFoundError)
    end

    it 'returns the order for a known id' do
      order = repo.place_order({ 'petId' => 1, 'quantity' => 2, 'status' => 'placed' })
      fetched = repo.find_order_by_id(order['id'])
      expect(fetched['id']).to eq(order['id'])
      expect(fetched['petId']).to eq(1)
      expect(fetched['quantity']).to eq(2)
      expect(fetched['status']).to eq('placed')
    end

    it 'converts string id to integer for lookup' do
      order = repo.place_order({ 'id' => 7, 'petId' => 1, 'quantity' => 1, 'status' => 'placed' })
      fetched = repo.find_order_by_id('7')
      expect(fetched['id']).to eq(7)
    end
  end

  describe '#delete_order' do
    it 'removes the order' do
      order = repo.place_order({ 'petId' => 1, 'quantity' => 1, 'status' => 'placed' })
      repo.delete_order(order['id'])
      expect { repo.find_order_by_id(order['id']) }.to raise_error(NotFoundError)
    end

    it 'converts string id to integer for deletion' do
      repo.place_order({ 'id' => 5, 'petId' => 1, 'quantity' => 1, 'status' => 'placed' })
      repo.delete_order('5')
      expect { repo.find_order_by_id(5) }.to raise_error(NotFoundError)
    end
  end

  # ---------------------------------------------------------------------------
  # User operations
  # ---------------------------------------------------------------------------

  let(:user) do
    {
      'id'         => 1,
      'username'   => 'testuser',
      'firstName'  => 'Test',
      'lastName'   => 'User',
      'email'      => 'test@example.com',
      'password'   => 'secret',
      'phone'      => '555-1234',
      'userStatus' => 0
    }
  end

  describe '#create_user' do
    it 'stores and retrieves the user' do
      repo.create_user(user)
      fetched = repo.find_user_by_username('testuser')
      expect(fetched['email']).to eq('test@example.com')
    end

    it 'preserves all user fields' do
      repo.create_user(user)
      found = repo.find_user_by_username('testuser')
      expect(found['id']).to eq(1)
      expect(found['username']).to eq('testuser')
      expect(found['firstName']).to eq('Test')
      expect(found['lastName']).to eq('User')
      expect(found['email']).to eq('test@example.com')
      expect(found['password']).to eq('secret')
      expect(found['phone']).to eq('555-1234')
      expect(found['userStatus']).to eq(0)
    end

    it 'accepts symbol keys for user fields' do
      sym_user = {
        id:         9,
        username:   'symuser',
        firstName:  'Sym',
        lastName:   'Person',
        email:      'sym@example.com',
        password:   'pw456',
        phone:      '999-0000',
        userStatus: 1
      }
      repo.create_user(sym_user)
      found = repo.find_user_by_username('symuser')
      expect(found['id']).to eq(9)
      expect(found['username']).to eq('symuser')
      expect(found['firstName']).to eq('Sym')
      expect(found['lastName']).to eq('Person')
      expect(found['email']).to eq('sym@example.com')
      expect(found['password']).to eq('pw456')
      expect(found['phone']).to eq('999-0000')
      expect(found['userStatus']).to eq(1)
    end

    it 'accepts snake_case first_name and last_name keys' do
      snake_user = {
        'id'         => 10,
        'username'   => 'snakeuser',
        'first_name' => 'Snake',
        'last_name'  => 'Case',
        'email'      => 'snake@example.com',
        'password'   => 'pw789',
        'phone'      => '111-2222',
        'userStatus' => 2
      }
      repo.create_user(snake_user)
      found = repo.find_user_by_username('snakeuser')
      expect(found['firstName']).to eq('Snake')
      expect(found['lastName']).to eq('Case')
    end

    it 'accepts snake_case user_status key' do
      snake_status_user = {
        'id'          => 11,
        'username'    => 'snakestatus',
        'firstName'   => 'A',
        'lastName'    => 'B',
        'email'       => 'a@b.com',
        'password'    => 'pw',
        'phone'       => '000',
        'user_status' => 3
      }
      repo.create_user(snake_status_user)
      found = repo.find_user_by_username('snakestatus')
      expect(found['userStatus']).to eq(3)
    end
  end

  describe '#create_users_with_list' do
    it 'stores all users' do
      users = [user, user.merge('username' => 'other', 'email' => 'other@example.com')]
      repo.create_users_with_list(users)
      expect(repo.find_user_by_username('testuser')).not_to be_nil
      expect(repo.find_user_by_username('other')).not_to be_nil
    end

    it 'stores each user individually accessible by username' do
      users = [
        user,
        user.merge('id' => 2, 'username' => 'second', 'email' => 'second@example.com')
      ]
      repo.create_users_with_list(users)
      expect(repo.find_user_by_username('testuser')['email']).to eq('test@example.com')
      expect(repo.find_user_by_username('second')['email']).to eq('second@example.com')
    end
  end

  describe '#find_user_by_username' do
    it 'raises NotFoundError for unknown username' do
      expect { repo.find_user_by_username('nobody') }.to raise_error(NotFoundError)
    end

    it 'looks up by exact username string' do
      repo.create_user(user)
      found = repo.find_user_by_username('testuser')
      expect(found['username']).to eq('testuser')
    end
  end

  describe '#update_user' do
    it 'updates user fields' do
      repo.create_user(user)
      repo.update_user('testuser', user.merge('email' => 'new@example.com'))
      expect(repo.find_user_by_username('testuser')['email']).to eq('new@example.com')
    end

    it 'raises NotFoundError for unknown username' do
      expect { repo.update_user('nobody', user) }.to raise_error(NotFoundError)
    end

    it 'preserves all fields after update' do
      repo.create_user(user)
      updated_data = {
        'id'         => 1,
        'username'   => 'testuser',
        'firstName'  => 'Updated',
        'lastName'   => 'Name',
        'email'      => 'updated@example.com',
        'password'   => 'newpass',
        'phone'      => '777-8888',
        'userStatus' => 5
      }
      repo.update_user('testuser', updated_data)
      found = repo.find_user_by_username('testuser')
      expect(found['firstName']).to eq('Updated')
      expect(found['lastName']).to eq('Name')
      expect(found['email']).to eq('updated@example.com')
      expect(found['password']).to eq('newpass')
      expect(found['phone']).to eq('777-8888')
      expect(found['userStatus']).to eq(5)
    end

    it 'always stores under the original username regardless of body' do
      repo.create_user(user)
      repo.update_user('testuser', user.merge('username' => 'other', 'email' => 'x@x.com'))
      found = repo.find_user_by_username('testuser')
      expect(found['email']).to eq('x@x.com')
    end
  end

  describe '#delete_user' do
    it 'removes the user' do
      repo.create_user(user)
      repo.delete_user('testuser')
      expect { repo.find_user_by_username('testuser') }.to raise_error(NotFoundError)
    end
  end

  describe '#login_user' do
    before { repo.create_user(user) }

    it 'returns true for correct credentials' do
      expect(repo.login_user('testuser', 'secret')).to be(true)
    end

    it 'returns false for wrong password' do
      expect(repo.login_user('testuser', 'wrong')).to be(false)
    end

    it 'returns false for unknown user' do
      expect(repo.login_user('nobody', 'secret')).to be(false)
    end

    it 'returns false not nil for unknown user' do
      result = repo.login_user('nobody', 'secret')
      expect(result).to eq(false)
    end

    it 'compares password exactly' do
      expect(repo.login_user('testuser', 'Secret')).to be(false)
      expect(repo.login_user('testuser', 'secret ')).to be(false)
    end
  end
end
