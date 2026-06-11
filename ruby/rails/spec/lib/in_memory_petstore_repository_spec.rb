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
  end

  describe '#find_pet_by_id' do
    it 'returns the pet for a known id' do
      added = repo.add_pet(pet)
      expect(repo.find_pet_by_id(added['id'])['name']).to eq('Fido')
    end

    it 'raises NotFoundError for unknown id' do
      expect { repo.find_pet_by_id(9999) }.to raise_error(NotFoundError)
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
      expect(repo.find_pets_by_tags([])).to be_empty
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
  end

  describe '#find_order_by_id' do
    it 'raises NotFoundError for unknown order' do
      expect { repo.find_order_by_id(9999) }.to raise_error(NotFoundError)
    end
  end

  describe '#delete_order' do
    it 'removes the order' do
      order = repo.place_order({ 'petId' => 1, 'quantity' => 1, 'status' => 'placed' })
      repo.delete_order(order['id'])
      expect { repo.find_order_by_id(order['id']) }.to raise_error(NotFoundError)
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
  end

  describe '#create_users_with_list' do
    it 'stores all users' do
      users = [user, user.merge('username' => 'other', 'email' => 'other@example.com')]
      repo.create_users_with_list(users)
      expect(repo.find_user_by_username('testuser')).not_to be_nil
      expect(repo.find_user_by_username('other')).not_to be_nil
    end
  end

  describe '#find_user_by_username' do
    it 'raises NotFoundError for unknown username' do
      expect { repo.find_user_by_username('nobody') }.to raise_error(NotFoundError)
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
  end
end
