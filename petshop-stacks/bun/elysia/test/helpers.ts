import type { Pet, Order, User } from '../models';

export function makePet(overrides: Partial<Pet> = {}): Pet {
  return {
    name: 'Fido',
    photoUrls: ['http://example.com/fido.jpg'],
    status: 'available',
    ...overrides,
  };
}

export function makeOrder(overrides: Partial<Order> = {}): Order {
  return {
    petId: 1,
    quantity: 1,
    status: 'placed',
    complete: false,
    ...overrides,
  };
}

export function makeUser(overrides: Partial<User> = {}): User {
  return {
    username: 'testuser',
    firstName: 'Test',
    lastName: 'User',
    email: 'test@example.com',
    password: 'secret',
    phone: '555-1234',
    userStatus: 1,
    ...overrides,
  };
}
